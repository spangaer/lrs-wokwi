#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::rmt::{
    Error as RmtError, PulseCode, Rmt, TxChannel, TxChannelConfig, TxChannelCreator,
};
use esp_hal::{prelude::*, timer::timg::TimerGroup};
use esp_hal_embassy as embassy;
use esp_println::{print, println};
use fixes::enable_lf_nl_mode;
use fugit::Duration as FDuration;
use once_cell::sync::Lazy;

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
            peripherals.GPIO8,
            TxChannelConfig {
                clk_divider: 1,
                ..TxChannelConfig::default()
            },
        )
        .unwrap();

    spawner.spawn(one_second_task()).unwrap();
    spawner.spawn(two_second_task()).unwrap();
    spawner.spawn(three_second_task()).unwrap();

    let mut count = 0;
    loop {
        println!("Main Task Count: {}", count);
        count += 1;

        channel = set_pixel(channel, Rgb::RED).await.unwrap();
        Timer::after(Duration::from_millis(1_000)).await;

        channel = set_pixel(channel, Rgb::GREEN).await.unwrap();
        Timer::after(Duration::from_millis(1_000)).await;

        channel = set_pixel(channel, Rgb::BLUE).await.unwrap();
        Timer::after(Duration::from_millis(1_000)).await;
    }
}

const PWM_PERIOD_NS: u32 = 1600;
const PWM_ZERO_HIGH_NS: u32 = 350;
const PWM_ZERO_LOW_NS: u32 = PWM_PERIOD_NS - PWM_ZERO_HIGH_NS;
const PWM_ONE_HIGH_NS: u32 = 900;
const PWM_ONE_LOW_NS: u32 = PWM_PERIOD_NS - PWM_ONE_HIGH_NS;
const PWM_RGB_BITS: u32 = 8 * 3;
const PWM_RGBSEND_US: u64 = (PWM_RGB_BITS * PWM_PERIOD_NS / 1000) as u64;

fn pulse_from_intervals(nanos_high: u32, nanos_low: u32) -> u32 {
    let ticks_high = FDuration::<u32, 1, RMT_TICK_RATE_HZ>::nanos(nanos_high).ticks() as u16;
    let ticks_low = FDuration::<u32, 1, RMT_TICK_RATE_HZ>::nanos(nanos_low).ticks() as u16;
    <u32 as PulseCode>::new(true, ticks_high, false, ticks_low)
}

// avoid "unsafe" by using once_cell_::sync::Lazy with critical-section feature

static PWM_ZERO: Lazy<u32> = Lazy::new(|| pulse_from_intervals(PWM_ZERO_HIGH_NS, PWM_ZERO_LOW_NS));
static PWM_ONE: Lazy<u32> = Lazy::new(|| pulse_from_intervals(PWM_ONE_HIGH_NS, PWM_ONE_LOW_NS));

async fn set_pixel<C: TxChannel>(channel: C, color: Rgb) -> Result<C, (RmtError, C)> {
    let bits: u32 = color.into();

    let mut pixel_data = [*PWM_ZERO; PWM_RGB_BITS as usize + 1];
    pixel_data[pixel_data.len() - 1] = PulseCode::empty(); // end marker

    for i in 0..PWM_RGB_BITS as usize {
        let bit = (bits >> i) & 1; // Extract the i-th bit

        if bit > 0 {
            pixel_data[i] = *PWM_ONE;
        }
    }

    // FIXME proper error handling
    let transaction = channel.transmit(&pixel_data).expect("transmit failed");

    Timer::after(Duration::from_micros(PWM_RGBSEND_US)).await;

    transaction.wait()
}

#[derive(Debug, Clone, Copy)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    const RED: Rgb = Self { r: 255, g: 0, b: 0 };
    const GREEN: Rgb = Self { r: 0, g: 255, b: 0 };
    const BLUE: Rgb = Self { r: 0, g: 0, b: 255 };

    #[allow(unused)]
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<Rgb> for u32 {
    /// Convert RGB to u32 color value
    fn from(rgb: Rgb) -> Self {
        ((rgb.r as u32) << 16) | ((rgb.g as u32) << 8) | rgb.b as u32
    }
}
