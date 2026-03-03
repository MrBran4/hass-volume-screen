pub const DISPLAY_WIDTH: usize = 240;
pub const DISPLAY_HEIGHT: usize = 240;

pub const WIFI_NETWORK: &str = env!("WF_SSID");
pub const WIFI_PASSWORD: &str = env!("WF_PASS");

pub const MQTT_CLIENT_ID: &str = env!("MQTT_CLIENT_ID");
pub const MQTT_HOST: &str = env!("MQTT_HOST");
pub const MQTT_USER: Option<&str> = option_env!("MQTT_USER");
pub const MQTT_PASS: Option<&str> = option_env!("MQTT_PASS");

pub const MQTT_TOPIC_BASE: &str = "amplifier/wxa50/#";
pub const MQTT_TOPIC_VOLUME: &str = "amplifier/wxa-50/volume";
pub const MQTT_TOPIC_INPUT: &str = "amplifier/wxa-50/input";
pub const MQTT_TOPIC_POWER: &str = "amplifier/wxa-50/power";
