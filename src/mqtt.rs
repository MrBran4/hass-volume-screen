use defmt::{error, info};
use embassy_futures::select::{Either, select};
use embassy_net::{Stack, dns::DnsQueryType, tcp::TcpSocket};
use embassy_time::Timer;
use rust_mqtt::{
    client::{client::MqttClient, client_config::ClientConfig},
    packet::v5::reason_codes::ReasonCode,
    utils::rng_generator::CountingRng,
};
use static_cell::StaticCell;

use crate::{
    channels::MQTT_STATUS,
    config::{
        self, MQTT_PASS, MQTT_TOPIC_BASE, MQTT_TOPIC_INPUT, MQTT_TOPIC_POWER, MQTT_TOPIC_VOLUME,
        MQTT_USER,
    },
};

static RX_BUFFER: StaticCell<[u8; 4096]> = StaticCell::new();
static TX_BUFFER: StaticCell<[u8; 4096]> = StaticCell::new();

mod receive_commands;

enum MqttError {
    Warning,
    Fatal,
}

#[derive(Debug, Clone)]
pub enum ConnectionState {
    Failed,
    Connecting,
    Connected,
}

/// Publishes updated readings to the MQTT broker, including the initial hass discovery message.
#[embassy_executor::task]
pub async fn worker(stack: Stack<'static>) {
    info!("[MQTT] Worker started");

    let rx_buffer = RX_BUFFER.init([0u8; 4096]);
    let tx_buffer = TX_BUFFER.init([0u8; 4096]);

    loop {
        {
            // Only claim the mutex temporarily
            let guard = MQTT_STATUS.lock().await;
            *guard.borrow_mut() = ConnectionState::Failed;
        }

        Timer::after_millis(500).await;

        let mut socket = TcpSocket::new(stack, rx_buffer, tx_buffer);

        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

        {
            // Only claim the mutex temporarily
            let guard = MQTT_STATUS.lock().await;
            *guard.borrow_mut() = ConnectionState::Connecting;
        }

        let address = match stack
            .dns_query(config::MQTT_HOST, DnsQueryType::A)
            .await
            .map(|a| a[0])
        {
            Ok(address) => address,
            Err(e) => {
                error!("[MQTT] DNS lookup error: {}", e);
                continue;
            }
        };

        let remote_endpoint = (address, 1883);
        info!("[MQTT] Connecting...");
        let connection = socket.connect(remote_endpoint).await;
        if let Err(e) = connection {
            error!("[MQTT] Connect error: {:?}", e);
            continue;
        }
        info!("mqtt connected!");

        let mut config = ClientConfig::new(
            rust_mqtt::client::client_config::MqttVersion::MQTTv5,
            CountingRng(20000),
        );
        config.add_max_subscribe_qos(rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS1);
        config.add_client_id(config::MQTT_CLIENT_ID);

        if let Some(username) = MQTT_USER {
            config.add_username(username);
        }

        if let Some(password) = MQTT_PASS {
            config.add_password(password);
        }

        config.max_packet_size = 100;
        let mut recv_buffer = [0; 8192];
        let mut write_buffer = [0; 8192];

        let mut client = MqttClient::<_, 5, _>::new(
            socket,
            &mut write_buffer,
            8192,
            &mut recv_buffer,
            512,
            config,
        );

        match client.connect_to_broker().await {
            Ok(()) => {}
            Err(mqtt_error) => match mqtt_error {
                ReasonCode::NetworkError => {
                    error!("[MQTT] Network error");
                    continue;
                }
                e => {
                    error!("[MQTT] other error: {}", e);
                    continue;
                }
            },
        }

        info!("[MQTT] Connected to broker");

        {
            // Only claim the mutex temporarily
            let guard = MQTT_STATUS.lock().await;
            *guard.borrow_mut() = ConnectionState::Connected;
        }

        // Subscribe to config topics
        if let Err(code) = client.subscribe_to_topic(MQTT_TOPIC_BASE).await {
            error!("[MQTT] Couldn't subscribe to state topic ({})", code);
        }

        loop {
            // Wait for a new reading, up to 1s and then send a ping otherwise.
            match select(client.receive_message(), Timer::after_secs(2)).await {
                Either::First(r) => {
                    if let Err(MqttError::Fatal) = receive_commands::process_incoming(r).await {
                        break;
                    }
                }
                Either::Second(_) => {
                    info!("[MQTT] Pinging broker");
                    if let Err(e) = client.send_ping().await {
                        error!("[MQTT] Couldn't ping broker: {}", e)
                    }
                    continue;
                }
            };
        }
    }
}
