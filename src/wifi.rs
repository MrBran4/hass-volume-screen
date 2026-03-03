use core::fmt::Write;
use defmt::{debug, error, info, warn};
use embassy_futures::select::{Either, select};
use embassy_net::Runner;
use embassy_time::Timer;
use esp_wifi::wifi::{
    ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiState,
};
use heapless::String;

use crate::{channels::WIFI_STATUS, config::WIFI_NETWORK};

pub enum ConnectionState {
    Failed,
    Connecting,
    Connected(ConnectionStats),
}

#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub rssi: i8,
    pub channel: u8,
    pub bssid: String<17>,
    pub ssid: String<32>,
}

impl ConnectionState {
    fn with_bssid(&mut self, mac: [u8; 6]) {
        match self {
            Self::Connected(state) => {
                write!(
                    &mut state.bssid,
                    "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                    mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
                )
                .unwrap();
            }
            _ => {}
        }
    }
}

#[embassy_executor::task]
pub async fn connection(mut controller: WifiController<'static>) {
    info!("[WIFI] start connection task");

    {
        // Only claim the mutex temporarily
        let guard = WIFI_STATUS.lock().await;
        *guard.borrow_mut() = ConnectionState::Connecting;
    }

    loop {
        // wait until we're no longer connected, and occasionally report rssi etc.
        loop {
            // Check if we're still connected
            if esp_wifi::wifi::wifi_state() != WifiState::StaConnected {
                // No longer connected - break this loop and reconnect.
                break;
            }

            {
                // Only claim the mutex temporarily
                let guard = WIFI_STATUS.lock().await;
                *guard.borrow_mut() = ConnectionState::Connected(ConnectionStats {
                    rssi: 0,
                    channel: 0,
                    bssid: String::new(),
                    ssid: String::new(),
                });
            }

            // Wait for 60 seconds, but also listen for disconnects in the meantime.
            match select(
                controller.wait_for_event(WifiEvent::StaDisconnected),
                Timer::after_secs(60),
            )
            .await
            {
                Either::First(_) => {
                    warn!("[WIFI] Get StaDisconnected event - breaking out of rssi loop");
                    break;
                }
                Either::Second(_) => {}
            }

            // Some time has passed and we're still connected.
            // Try an AP scan and report network info.
            let ap_list = match controller.scan_n_async::<50>().await {
                Ok(aps) => aps,
                Err(e) => {
                    warn!("[WIFI] Couldn't search: {}", e);
                    continue;
                }
            };

            debug!("[WIFI] Finished AP search");
            debug!("[WIFI] Found {} APs: {}", ap_list.1, ap_list.0);

            let Some(our_ap) = ap_list.0.iter().find(|el| el.ssid == WIFI_NETWORK) else {
                warn!("[WIFI] Our AP wasn't in the search results list");
                continue;
            };

            {
                // Only claim the mutex temporarily
                let guard = WIFI_STATUS.lock().await;
                let mut inner = guard.borrow_mut();
                *inner = ConnectionState::Connected(ConnectionStats {
                    rssi: our_ap.signal_strength,
                    channel: our_ap.channel,
                    ssid: our_ap.ssid.clone(),
                    bssid: String::new(),
                });
                (*inner).with_bssid(our_ap.bssid);
            }
        }

        warn!("[WIFI] No longer connected! Reconnecting...");
        {
            // Only claim the mutex temporarily
            let guard = WIFI_STATUS.lock().await;
            *guard.borrow_mut() = ConnectionState::Failed;
            info!("[WIFI] Published disconnection state");
        }

        match controller.stop_async().await {
            Ok(()) => info!("[WIFI] Stopped wifi controller"),
            Err(e) => error!("[WIFI] Couldn't stop wifi controller: {}", e),
        };

        Timer::after_secs(10).await;

        match controller.is_started() {
            Ok(true) => {
                info!("[WIFI] Controller has started")
            }
            Err(e) => {
                error!("[WIFI] Couldn't check if Wifi is started: {}", e)
            }
            Ok(false) => {
                info!("[WIFI] Controller has not started, starting it now");

                let client_config = Configuration::Client(ClientConfiguration {
                    ssid: crate::config::WIFI_NETWORK.try_into().unwrap(),
                    password: crate::config::WIFI_PASSWORD.try_into().unwrap(),
                    ..Default::default()
                });

                info!("[WIFI] Setting configuration...");
                controller.set_configuration(&client_config).unwrap();

                info!("[WIFI] Starting wifi...");
                controller.start_async().await.unwrap();

                info!("[WIFI] Wifi started!");
            }
        }

        info!("[WIFI] About to connect...");
        loop {
            match controller.connect_async().await {
                Ok(_) => break,
                Err(e) => {
                    warn!("[WIFI] Failed to connect to wifi: {:?}", e);

                    {
                        // Only claim the mutex temporarily
                        let guard = WIFI_STATUS.lock().await;
                        *guard.borrow_mut() = ConnectionState::Failed;
                    }

                    Timer::after_secs(2).await;
                }
            }
            info!("[WIFI] Wifi connected!")
        }
    }
}

#[embassy_executor::task]
pub async fn net_task(mut runner: Runner<'static, WifiDevice<'static>>) {
    runner.run().await
}
