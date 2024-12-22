#![no_std]
#![no_main]

use core::iter;

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::rmt::{Rmt, TxChannelConfig, TxChannelCreator};
use esp_hal::{prelude::*, timer::timg::TimerGroup};
use esp_hal_embassy as embassy;
use esp_println::{print, println};
use fixes::enable_lf_nl_mode;
use neopixel::{PixelSender, Rgb};

#[embassy_executor::task]
async fn one_second_task() {
    let mut count = 0;
    loop {
        println!("1s Task Count: {}", count);
        count += 1;
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

#[embassy_executor::task]
async fn two_second_task() {
    let mut count = 0;
    loop {
        println!("2s Task Count: {}", count);
        count += 1;
        Timer::after(Duration::from_millis(2_000)).await;
    }
}

#[embassy_executor::task]
async fn three_second_task() {
    let mut count = 0;
    loop {
        println!("3s Task Count: {}", count);
        count += 1;
        Timer::after(Duration::from_millis(3_000)).await;
    }
}

/// For Neopixel PWM driving a 20 MHz resolution is technically enough,
/// but this will work with any multiple, up to 80 Mhz. esp_hal refuses
/// higher. esp32(s) requires 80 Mhz btw. For esp32-c, why not be kind
/// to Wokwi, right?
const RMT_TICK_RATE_HZ: u32 = 20_000_000;

#[main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    enable_lf_nl_mode!();

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    embassy::init(timg0.timer0);

    let rmt = Rmt::new(peripherals.RMT, RMT_TICK_RATE_HZ.Hz()).unwrap();

    let mut channel = rmt
        .channel0
        .configure(
            peripherals.GPIO6,
            TxChannelConfig {
                clk_divider: 1,
                ..TxChannelConfig::default()
            },
        )
        .unwrap();

    let sender = PixelSender::<RMT_TICK_RATE_HZ>::new();

    spawner.spawn(one_second_task()).unwrap();
    spawner.spawn(two_second_task()).unwrap();
    spawner.spawn(three_second_task()).unwrap();

    let mut count = 0;
    loop {
        println!("Main Task Count: {}", count);
        count += 1;

        channel = sender
            .set_pixels(channel, iter::once(Rgb::BLACK))
            .await
            .unwrap();
        Timer::after(Duration::from_millis(1_000)).await;

        channel = sender.set_pixel(channel, Rgb::PURPLE).await.unwrap();
        Timer::after(Duration::from_millis(1_000)).await;

        channel = sender.set_pixel(channel, Rgb::YELLOW).await.unwrap();
        Timer::after(Duration::from_millis(1_000)).await;
    }
}
