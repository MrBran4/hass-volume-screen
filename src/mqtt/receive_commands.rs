use defmt::{error, info, warn};
use rust_mqtt::packet::v5::reason_codes::ReasonCode;

use crate::{
    channels::LATEST_STATE,
    config::{MQTT_TOPIC_INPUT, MQTT_TOPIC_POWER, MQTT_TOPIC_VOLUME},
    wxa50,
};

use super::MqttError;

/// Process a received message
pub async fn process_incoming(msg: Result<(&str, &[u8]), ReasonCode>) -> Result<(), MqttError> {
    match msg {
        Ok((MQTT_TOPIC_VOLUME, p)) => process_volume_msg(p).await,
        Ok((MQTT_TOPIC_INPUT, p)) => process_input_msg(p).await,
        Ok((MQTT_TOPIC_POWER, p)) => process_power_msg(p).await,
        Ok((topic, _)) => {
            error!("Received message on unexpected topic {}", topic);
            Ok(())
        }
        Err(ReasonCode::NetworkError) => {
            error!("MQTT Listening failed with a network error");
            Err(MqttError::Fatal)
        }
        Err(code) => {
            error!("MQTT Listened failed with MQTT code: {}", code);
            Err(MqttError::Warning)
        }
    }
}

async fn process_volume_msg(payload_bytes: &[u8]) -> Result<(), MqttError> {
    info!("Received new Volume message: {}", payload_bytes);

    // Try parsing it as a number
    let Ok(s) = core::str::from_utf8(payload_bytes) else {
        warn!("Volume message wasn't valid utf-8 so it won't be processed");
        return Err(MqttError::Warning);
    };

    let Ok(new_val) = s.parse::<f32>() else {
        warn!("Volume message wasn't a valid f32 so it'll be ignored");
        return Err(MqttError::Warning);
    };

    info!("New volume will be: {}", new_val);

    {
        // Only claim the mutex temporarily
        let guard = LATEST_STATE.lock().await;
        (*guard.borrow_mut()).volume = new_val;
    }

    Ok(())
}

async fn process_power_msg(payload_bytes: &[u8]) -> Result<(), MqttError> {
    info!("Received new Power message: {}", payload_bytes);

    // Try parsing it as a number
    let Ok(s) = core::str::from_utf8(payload_bytes) else {
        warn!("Power message wasn't valid utf-8 so it won't be processed");
        return Err(MqttError::Warning);
    };

    let new_val = match s {
        "ON" => wxa50::Power::On,
        _ => wxa50::Power::Off,
    };

    info!("New power will be: {}", new_val);

    {
        // Only claim the mutex temporarily
        let guard = LATEST_STATE.lock().await;
        (*guard.borrow_mut()).power = new_val;
    }

    Ok(())
}

async fn process_input_msg(payload_bytes: &[u8]) -> Result<(), MqttError> {
    info!("Received new Input message: {}", payload_bytes);

    // Try parsing it as a number
    let Ok(s) = core::str::from_utf8(payload_bytes) else {
        warn!("Input message wasn't valid utf-8 so it won't be processed");
        return Err(MqttError::Warning);
    };

    let new_val = match s {
        "TV (Optical)" | "Optical" => wxa50::Input::Optical,
        "TV (Wired)" | "Wired" => wxa50::Input::Wired,
        "AirPlay" | "airplay" => wxa50::Input::AirPlay,
        _ => wxa50::Input::Unknown,
    };

    info!("New input will be: {}", new_val);

    {
        // Only claim the mutex temporarily
        let guard = LATEST_STATE.lock().await;
        (*guard.borrow_mut()).input = new_val;
    }

    Ok(())
}
