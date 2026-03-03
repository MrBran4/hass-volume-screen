use embassy_time::Timer;
use embedded_graphics::{image::Image, pixelcolor::Rgb565, prelude::*};
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    Blocking,
    gpio::Output,
    ledc::{self, LowSpeed, channel::ChannelIFace},
    spi::master::SpiDmaBus,
};
use mipidsi::{interface::SpiInterface, models::GC9A01};
use u8g2_fonts::types::HorizontalAlignment;

use crate::{
    channels::{LATEST_STATE, MQTT_STATUS, WIFI_STATUS},
    ui::images::{
        RING_0_10, RING_10_20, RING_20_30, RING_30_40, RING_40_50, RING_50_60, RING_60_70,
        RING_70_80, RING_80_90, RING_90_100, RING_OFF, SPINNER_1, SPINNER_2, SPINNER_3, SPINNER_4,
        SPINNER_5, SPINNER_6, SPINNER_7, SPINNER_8, STARTUP_MQTT, STARTUP_WIFI,
    },
    wxa50,
};

mod drawables;
mod images;

type Backlight = ledc::channel::Channel<'static, LowSpeed>;

type BacklightTimer = ledc::timer::Timer<'static, LowSpeed>;

type Display = mipidsi::Display<
    SpiInterface<
        'static,
        ExclusiveDevice<SpiDmaBus<'static, Blocking>, Output<'static>, esp_hal::delay::Delay>,
        Output<'static>,
    >,
    GC9A01,
    Output<'static>,
>;

#[derive(PartialEq, Eq)]
enum CurrentBackdrop {
    Blank,
    StartupWifi,
    StartupMqtt,
    RingOff,
    Ring0_10,
    Ring10_20,
    Ring20_30,
    Ring30_40,
    Ring40_50,
    Ring50_60,
    Ring60_70,
    Ring70_80,
    Ring80_90,
    Ring90_100,
}

#[derive(Debug, PartialEq)]
struct CoreState {
    wifi_ok: bool,
    mqtt_ok: bool,
    volume: f32,
    input: wxa50::Input,
    power: wxa50::Power,
}

async fn get_core_state() -> CoreState {
    let (volume, power, input) = {
        let lock = LATEST_STATE.lock().await;
        let rc = lock.borrow();
        (rc.volume, rc.power.clone(), rc.input.clone())
    };

    CoreState {
        wifi_ok: matches!(
            *WIFI_STATUS.lock().await.borrow(),
            crate::wifi::ConnectionState::Connected(_)
        ),
        mqtt_ok: matches!(
            *MQTT_STATUS.lock().await.borrow(),
            crate::mqtt::ConnectionState::Connected
        ),
        volume,
        input,
        power,
    }
}

#[embassy_executor::task]
pub async fn worker(
    mut backlight: Backlight,
    backlight_timer: BacklightTimer,
    mut display: Display,
) {
    let mut current_backdrop = CurrentBackdrop::Blank;
    _ = display.clear(Rgb565::BLACK);

    let mut last_state = CoreState {
        wifi_ok: false,
        mqtt_ok: false,
        volume: 0.0,
        input: wxa50::Input::Unknown,
        power: wxa50::Power::Off,
    };

    let mut last_significant_change = embassy_time::Instant::now();

    loop {
        // Did the UI change this particular iteration? (e.g. do we need to re-render)
        let mut rerender = false;

        let new_state = get_core_state().await;

        let now = embassy_time::Instant::now();

        if new_state.power != last_state.power
            || new_state.wifi_ok != last_state.wifi_ok
            || new_state.mqtt_ok != last_state.mqtt_ok
            || new_state.volume != last_state.volume
            || new_state.input != last_state.input
        {
            last_significant_change = now;
            last_state = new_state;
            rerender = true;
        }

        // Backlight brightness is on for 5s after a change, or, permanently if there's no wifi or mqtt.
        let display_brightness = match (
            &last_state.power,
            (now - last_significant_change).as_secs(),
            last_state.wifi_ok,
            last_state.mqtt_ok,
        ) {
            // Either wifi or mqtt erroring,  -> on at 25%
            (_, _, false, _) | (_, _, _, false) => 25,

            // On + changed in the last 5s -> On at 75%
            (wxa50::Power::On, ..=5, _, _) => 75,

            // Otherwise off
            _ => 0,
        };

        backlight
            .configure(ledc::channel::config::Config {
                timer: &backlight_timer,
                duty_pct: display_brightness,
                pin_config: ledc::channel::config::PinConfig::PushPull,
            })
            .unwrap();

        if !last_state.wifi_ok {
            play_wifi_connect(&mut display, &mut current_backdrop).await;
            continue;
        }

        if !last_state.mqtt_ok {
            play_mqtt_connect(&mut display, &mut current_backdrop).await;
            continue;
        }

        // We know we have valid readings now, so just render them
        if rerender {
            render_current_state(
                &mut display,
                &mut current_backdrop,
                &last_state.power,
                &last_state.input,
                &last_state.volume,
            )
            .await;
        }

        // Wait a short while before checking for more readings for other threads to do work.
        Timer::after_millis(250).await;
    }
}

async fn play_startup_spinner(display: &mut Display) {
    let spinner_frame_1 = Image::new(&SPINNER_1, (115, 165).into());
    let spinner_frame_2 = Image::new(&SPINNER_2, (115, 165).into());
    let spinner_frame_3 = Image::new(&SPINNER_3, (115, 165).into());
    let spinner_frame_4 = Image::new(&SPINNER_4, (115, 165).into());
    let spinner_frame_5 = Image::new(&SPINNER_5, (115, 165).into());
    let spinner_frame_6 = Image::new(&SPINNER_6, (115, 165).into());
    let spinner_frame_7 = Image::new(&SPINNER_7, (115, 165).into());
    let spinner_frame_8 = Image::new(&SPINNER_8, (115, 165).into());

    _ = spinner_frame_1.draw(display);
    Timer::after_millis(100).await;
    _ = spinner_frame_2.draw(display);
    Timer::after_millis(100).await;
    _ = spinner_frame_3.draw(display);
    Timer::after_millis(100).await;
    _ = spinner_frame_4.draw(display);
    Timer::after_millis(100).await;
    _ = spinner_frame_5.draw(display);
    Timer::after_millis(100).await;
    _ = spinner_frame_6.draw(display);
    Timer::after_millis(100).await;
    _ = spinner_frame_7.draw(display);
    Timer::after_millis(100).await;
    _ = spinner_frame_8.draw(display);
    Timer::after_millis(100).await;
}

async fn play_wifi_connect(display: &mut Display, current_backdrop: &mut CurrentBackdrop) {
    if !matches!(current_backdrop, CurrentBackdrop::StartupWifi) {
        let backdrop = Image::new(&STARTUP_WIFI, Point::zero());
        _ = backdrop.draw(display);
        *current_backdrop = CurrentBackdrop::StartupWifi
    }

    play_startup_spinner(display).await;
}

async fn play_mqtt_connect(display: &mut Display, current_backdrop: &mut CurrentBackdrop) {
    if !matches!(current_backdrop, CurrentBackdrop::StartupMqtt) {
        let backdrop = Image::new(&STARTUP_MQTT, Point::zero());
        _ = backdrop.draw(display);
        *current_backdrop = CurrentBackdrop::StartupMqtt
    }

    play_startup_spinner(display).await;
}

async fn render_current_state(
    display: &mut Display,
    current_backdrop: &mut CurrentBackdrop,
    power: &wxa50::Power,
    input: &wxa50::Input,
    volume: &f32,
) {
    let desired_backdrop = match (power, volume) {
        (wxa50::Power::Off, _) => CurrentBackdrop::RingOff,
        (wxa50::Power::On, ..=10.0) => CurrentBackdrop::Ring0_10,
        (wxa50::Power::On, ..=20.0) => CurrentBackdrop::Ring10_20,
        (wxa50::Power::On, ..=30.0) => CurrentBackdrop::Ring20_30,
        (wxa50::Power::On, ..=40.0) => CurrentBackdrop::Ring30_40,
        (wxa50::Power::On, ..=50.0) => CurrentBackdrop::Ring40_50,
        (wxa50::Power::On, ..=60.0) => CurrentBackdrop::Ring50_60,
        (wxa50::Power::On, ..=70.0) => CurrentBackdrop::Ring60_70,
        (wxa50::Power::On, ..=80.0) => CurrentBackdrop::Ring70_80,
        (wxa50::Power::On, ..=90.0) => CurrentBackdrop::Ring80_90,
        (wxa50::Power::On, 90.0..) => CurrentBackdrop::Ring90_100,
        (wxa50::Power::On, _) => CurrentBackdrop::Ring0_10,
    };

    let backdrop = match desired_backdrop {
        CurrentBackdrop::Blank => &RING_OFF,
        CurrentBackdrop::RingOff => &RING_OFF,
        CurrentBackdrop::Ring0_10 => &RING_0_10,
        CurrentBackdrop::Ring10_20 => &RING_10_20,
        CurrentBackdrop::Ring20_30 => &RING_20_30,
        CurrentBackdrop::Ring30_40 => &RING_30_40,
        CurrentBackdrop::Ring40_50 => &RING_40_50,
        CurrentBackdrop::Ring50_60 => &RING_50_60,
        CurrentBackdrop::Ring60_70 => &RING_60_70,
        CurrentBackdrop::Ring70_80 => &RING_70_80,
        CurrentBackdrop::Ring80_90 => &RING_80_90,
        CurrentBackdrop::Ring90_100 => &RING_90_100,
        _ => &RING_OFF,
    };

    if current_backdrop != &desired_backdrop {
        _ = backdrop.draw(display);
        *current_backdrop = desired_backdrop;
    }

    // Draw a black circle over the previous readings
    drawables::draw_circle(display, (120, 120).into(), 176).await;

    // Draw the new percentage
    drawables::draw_percentage(
        display,
        *volume,
        (120, 87).into(),
        HorizontalAlignment::Center,
    )
    .await;

    // Draw the new input string
    drawables::draw_text(
        display,
        match input {
            wxa50::Input::Optical => "Optical",
            wxa50::Input::Wired => "Aux",
            wxa50::Input::AirPlay => "AirPlay",
            wxa50::Input::Unknown => "Unknown",
        },
        (120, 142).into(),
        HorizontalAlignment::Center,
    )
    .await;
}
