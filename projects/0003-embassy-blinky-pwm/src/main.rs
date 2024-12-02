#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::gpio::{Level, Output};
use esp_hal::rmt::{PulseCode, Rmt, TxChannelAsync, TxChannelConfig, TxChannelCreatorAsync};
use esp_hal::{prelude::*, timer::timg::TimerGroup};
use esp_hal_embassy as embassy;
use esp_println::{print, println};
use fugit::Duration as FDuration;

fn is_done() -> bool {
    let rmt = unsafe { &*esp_hal::peripherals::RMT::PTR };
    rmt.int_raw().read().ch1_tx_end().bit()
}

#[embassy_executor::task]
async fn one_second_task(mut pin: Output<'static>) {
    let mut count = 0;
    loop {
        esp_println::println!("1s Task Count: {}", count);
        count += 1;
        Timer::after(Duration::from_millis(1_000)).await;
        match pin.output_level() {
            Level::High => pin.set_low(),
            Level::Low => pin.set_high(),
        };
        println!("is done {}", is_done()) // doesn't turn true?
    }
}

#[embassy_executor::task]
async fn two_second_task() {
    let mut count = 0;
    loop {
        esp_println::println!("2s Task Count: {}", count);
        count += 1;
        Timer::after(Duration::from_millis(2_000)).await;
    }
}

#[embassy_executor::task]
async fn three_second_task() {
    let mut count = 0;
    loop {
        esp_println::println!("3s Task Count: {}", count);
        count += 1;
        Timer::after(Duration::from_millis(3_000)).await;
    }
}

const RAW_CLOCK_HZ: u32 = 20_000_000;

fn pulse_from_intervals(nanos_high: u32, nanos_low: u32) -> u32 {
    let ticks_high = FDuration::<u32, 1, RAW_CLOCK_HZ>::nanos(nanos_high).ticks() as u16;
    let ticks_low = FDuration::<u32, 1, RAW_CLOCK_HZ>::nanos(nanos_low).ticks() as u16;
    <u32 as PulseCode>::new(true, ticks_high, false, ticks_low)
}

const PWM_PERIOD_NS: u32 = 1600;
const PWM_ZERO_HIGH_NS: u32 = 350;
const PWM_ZERO_LOW_NS: u32 = PWM_PERIOD_NS - PWM_ZERO_HIGH_NS;
const PWM_ONE_HIGH_NS: u32 = 900;
const PWM_ONE_LOW_NS: u32 = PWM_PERIOD_NS - PWM_ONE_HIGH_NS;

#[main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    embassy::init(timg0.timer0);

    // This line is for Wokwi only so that the console output is formatted correctly
    print!("\x1b[20h");
    println!("Hello");

    let rmt = Rmt::new(peripherals.RMT, RAW_CLOCK_HZ.Hz())
        .unwrap()
        .into_async();

    let mut channel = rmt
        .channel1 // on channel0 it blocks the whole executor
        .configure(
            peripherals.GPIO8,
            TxChannelConfig {
                clk_divider: 1,
                ..TxChannelConfig::default()
            },
        )
        .unwrap();

    let pwm_zero = pulse_from_intervals(PWM_ZERO_HIGH_NS, PWM_ZERO_LOW_NS);
    let pwm_one = pulse_from_intervals(PWM_ONE_HIGH_NS, PWM_ONE_LOW_NS);
    println!("zero: {:b} ; one: {:b}", pwm_zero, pwm_one);

    let ledpin: Output<'static> = Output::new(peripherals.GPIO18, Level::Low);
    spawner.spawn(one_second_task(ledpin)).unwrap();
    spawner.spawn(two_second_task()).unwrap();
    spawner.spawn(three_second_task()).unwrap();

    let mut white = [pwm_one; 25];
    white[white.len() - 1] = PulseCode::empty(); // end marker

    // for i in 0..=8 {
    //     white[i] = pwm_zero;
    // }
    // white[white.len() - 1] = PulseCode::empty(); // end marker

    // channel.transmit(&white);

    // match channel.transmit(&white).await {
    //     Err(e) => println!("{:?}", e),
    //     _ => println!("transmitted"),
    // };

    let mut count = 0;
    loop {
        esp_println::println!("Main Task Count: {}", count);
        count += 1;

        // never returns :(
        match channel.transmit(&white).await {
            Err(e) => println!("{:?}", e),
            _ => println!("transmitted"),
        };

        Timer::after(Duration::from_millis(3_000)).await;
    }
}
