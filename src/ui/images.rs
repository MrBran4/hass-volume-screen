use embedded_graphics::{
    image::{ImageRaw, ImageRawLE},
    pixelcolor::Rgb565,
};

// Include the raw image data at compile time
pub const RING_OFF: ImageRawLE<Rgb565> =
    ImageRaw::new(include_bytes!("images/percentage-0.raw"), 240);
pub const RING_0_10: ImageRawLE<Rgb565> =
    ImageRaw::new(include_bytes!("images/percentage-10.raw"), 240);
pub const RING_10_20: ImageRawLE<Rgb565> =
    ImageRaw::new(include_bytes!("images/percentage-20.raw"), 240);
pub const RING_20_30: ImageRawLE<Rgb565> =
    ImageRaw::new(include_bytes!("images/percentage-30.raw"), 240);
pub const RING_30_40: ImageRawLE<Rgb565> =
    ImageRaw::new(include_bytes!("images/percentage-40.raw"), 240);
pub const RING_40_50: ImageRawLE<Rgb565> =
    ImageRaw::new(include_bytes!("images/percentage-50.raw"), 240);
pub const RING_50_60: ImageRawLE<Rgb565> =
    ImageRaw::new(include_bytes!("images/percentage-60.raw"), 240);
pub const RING_60_70: ImageRawLE<Rgb565> =
    ImageRaw::new(include_bytes!("images/percentage-70.raw"), 240);
pub const RING_70_80: ImageRawLE<Rgb565> =
    ImageRaw::new(include_bytes!("images/percentage-80.raw"), 240);
pub const RING_80_90: ImageRawLE<Rgb565> =
    ImageRaw::new(include_bytes!("images/percentage-90.raw"), 240);
pub const RING_90_100: ImageRawLE<Rgb565> =
    ImageRaw::new(include_bytes!("images/percentage-100.raw"), 240);

pub const STARTUP_WIFI: ImageRawLE<Rgb565> =
    ImageRaw::new(include_bytes!("images/startup-wifi.raw"), 172);

pub const STARTUP_MQTT: ImageRawLE<Rgb565> =
    ImageRaw::new(include_bytes!("images/startup-mqtt.raw"), 172);

pub const SPINNER_1: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("images/spinner-1.raw"), 10);
pub const SPINNER_2: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("images/spinner-2.raw"), 10);
pub const SPINNER_3: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("images/spinner-3.raw"), 10);
pub const SPINNER_4: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("images/spinner-4.raw"), 10);
pub const SPINNER_5: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("images/spinner-5.raw"), 10);
pub const SPINNER_6: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("images/spinner-6.raw"), 10);
pub const SPINNER_7: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("images/spinner-7.raw"), 10);
pub const SPINNER_8: ImageRawLE<Rgb565> = ImageRaw::new(include_bytes!("images/spinner-8.raw"), 10);
