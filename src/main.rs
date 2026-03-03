#![no_std]
#![no_main]
#![feature(type_alias_impl_trait, impl_trait_in_assoc_type, auto_traits)]

use crate::config::{DISPLAY_HEIGHT, DISPLAY_WIDTH};
use alloc::boxed::Box;
use config::MQTT_CLIENT_ID;
use defmt::info;
use embassy_executor::Spawner;
use embassy_net::{DhcpConfig, StackResources};
use embassy_time::Timer;
use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    dma::{DmaRxBuf, DmaTxBuf},
    dma_buffers,
    gpio::{Level, Output, OutputConfig},
    ledc::{self, LSGlobalClkSource, Ledc, LowSpeed, timer::TimerIFace},
    rng::Rng,
    spi::{self, master::Spi},
    time::Rate,
    timer::timg::TimerGroup,
};
use esp_wifi::EspWifiController;
use mipidsi::options::{ColorInversion, Orientation, Rotation};

use {esp_backtrace as _, esp_println as _};

extern crate alloc;

pub mod channels;
pub mod config;
pub mod mqtt;
pub mod ui;
pub mod wifi;
pub mod wxa50;

/// esp_backtrace uses this as its halt implementation.
/// Instead of loop{} (which is not useful) we reset the chip (which is more useful)
#[unsafe(no_mangle)]
pub extern "Rust" fn custom_halt() -> ! {
    esp_hal::system::software_reset()
}

macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let p = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 256 * 1024);

    info!("Embassy initialized!");

    let timer_group = TimerGroup::new(p.TIMG0);
    let mut rng = Rng::new(p.RNG);

    // --- START DISPLAY
    info!("Initializing display...");

    // Set up DMA for display
    let dma_channel = p.DMA_CH1;
    let dma_buffs = dma_buffers!(16000);
    let rx_buf = DmaRxBuf::new(dma_buffs.1, dma_buffs.0).unwrap();
    let tx_buf = DmaTxBuf::new(dma_buffs.3, dma_buffs.2).unwrap();

    let spi = Spi::new(
        p.SPI2,
        spi::master::Config::default()
            .with_frequency(Rate::from_mhz(20))
            .with_mode(spi::Mode::_0),
    )
    .unwrap();

    let spi = spi
        .with_sck(p.GPIO7)
        .with_mosi(p.GPIO6)
        .with_dma(dma_channel)
        .with_buffers(rx_buf, tx_buf);

    let cs_output = Output::new(p.GPIO14, Level::High, OutputConfig::default());
    let spi_delay = Delay::new();
    let spi_device = ExclusiveDevice::new(spi, cs_output, spi_delay).unwrap();

    let lcd_dc = Output::new(p.GPIO15, Level::Low, OutputConfig::default());
    let buffer: &'static mut [u8; 512] = Box::leak(Box::new([0_u8; 512]));
    let di = mipidsi::interface::SpiInterface::new(spi_device, lcd_dc, buffer);

    let mut display_delay = Delay::new();
    display_delay.delay_micros(500);

    let reset = Output::new(p.GPIO21, Level::Low, OutputConfig::default());
    let display = mipidsi::Builder::new(mipidsi::models::ST7789, di)
        .reset_pin(reset)
        .display_size(DISPLAY_WIDTH as u16, DISPLAY_HEIGHT as u16)
        .display_offset(34, 0)
        .orientation(Orientation::new().rotate(Rotation::Deg0))
        .invert_colors(ColorInversion::Inverted)
        .init(&mut display_delay)
        .unwrap();

    let mut ledc = Ledc::new(p.LEDC);
    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);
    let mut backlight_timer = ledc.timer::<LowSpeed>(ledc::timer::Number::Timer2);
    backlight_timer
        .configure(ledc::timer::config::Config {
            duty: ledc::timer::config::Duty::Duty8Bit,
            frequency: Rate::from_hz(30), // Lower frequency, less heat
            clock_source: ledc::timer::LSClockSource::APBClk,
        })
        .unwrap();

    let backlight_channel = ledc.channel(ledc::channel::Number::Channel2, p.GPIO22);

    info!("[MAIN] Starting ui task...");
    spawner
        .spawn(ui::worker(backlight_channel, backlight_timer, display))
        .ok();

    // --- START WIFI ---

    let esp_wifi_ctrl = &*mk_static!(
        EspWifiController<'static>,
        esp_wifi::init(timer_group.timer0, rng, p.RADIO_CLK).unwrap()
    );

    let (mut controller, interfaces) =
        esp_wifi::wifi::new(esp_wifi_ctrl, p.WIFI).expect("couldn't create esp_wifi");

    controller
        .set_power_saving(esp_wifi::config::PowerSaveMode::None)
        .expect("couldn't set wifi power save mode");

    let wifi_interface = interfaces.sta;
    let mut dhcp_config = DhcpConfig::default();
    dhcp_config.hostname = Some(heapless::String::try_from(MQTT_CLIENT_ID).unwrap());
    let net_config = embassy_net::Config::dhcpv4(dhcp_config);

    let seed = (rng.random() as u64) << 32 | rng.random() as u64;

    // Init network stack
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        net_config,
        mk_static!(StackResources<3>, StackResources::<3>::new()),
        seed,
    );

    let timg1 = TimerGroup::new(p.TIMG1);
    esp_hal_embassy::init(timg1.timer0);

    // Spawn network stuff
    spawner.spawn(wifi::connection(controller)).ok();
    spawner.spawn(wifi::net_task(runner)).ok();

    info!("Waiting for WiFi signal...");
    stack.wait_link_up().await;
    info!("Connected to WiFi!");

    info!("Waiting for network config via DHCP...");
    stack.wait_config_up().await;
    info!("Got valid IP configuration from DHCP");

    // --- START SERVICES ---

    info!("[MAIN] Starting MQTT task...");
    spawner.spawn(mqtt::worker(stack)).ok();

    loop {
        Timer::after_secs(30).await;
        info!("Main loop")
    }
}
