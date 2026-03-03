use core::cell::RefCell;

use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

use crate::{mqtt, wifi, wxa50};

pub static LATEST_STATE: Mutex<CriticalSectionRawMutex, RefCell<wxa50::State>> =
    Mutex::new(RefCell::new(wxa50::State {
        volume: 0.0,
        power: wxa50::Power::Off,
        input: wxa50::Input::Unknown,
    }));

pub static WIFI_STATUS: Mutex<CriticalSectionRawMutex, RefCell<wifi::ConnectionState>> =
    Mutex::new(RefCell::new(wifi::ConnectionState::Connecting));

pub static MQTT_STATUS: Mutex<CriticalSectionRawMutex, RefCell<mqtt::ConnectionState>> =
    Mutex::new(RefCell::new(mqtt::ConnectionState::Connecting));
