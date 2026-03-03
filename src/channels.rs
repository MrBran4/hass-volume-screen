use core::cell::RefCell;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

use crate::{mqtt, wifi, wxa50};

pub static LATEST_STATE: Mutex<CriticalSectionRawMutex, RefCell<wxa50::State>> =
    Mutex::new(RefCell::new(wxa50::State {
        volume: 0.0,
        power: wxa50::Power::Off,
        input: wxa50::Input::Unknown,
    }));

// Create channel for initialisation messages to be sent to the UI.
pub static INITIALISATION_UI: embassy_sync::channel::Channel<
    CriticalSectionRawMutex,
    InitialisationState,
    10,
> = embassy_sync::channel::Channel::new();

pub static WIFI_STATUS: Mutex<CriticalSectionRawMutex, RefCell<wifi::ConnectionState>> =
    Mutex::new(RefCell::new(wifi::ConnectionState::Connecting));

pub static MQTT_STATUS: Mutex<CriticalSectionRawMutex, RefCell<mqtt::ConnectionState>> =
    Mutex::new(RefCell::new(mqtt::ConnectionState::Connecting));

#[derive(PartialEq, Eq)]
pub enum InitialisationState {
    NoWifi,
    NoMqtt,
    NoSensor,
    Success,
}
