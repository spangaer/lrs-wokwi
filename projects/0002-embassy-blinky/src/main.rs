#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::gpio::{Level, Output};

// use embedded_hal::digital::OutputPin;
use esp_hal::{prelude::*, timer::timg::TimerGroup};
use esp_hal_embassy as embassy;
use esp_println::print;

#[embassy_executor::task]
async fn one_second_task(mut pin: Output<'static>) {
    let mut count = 0;
    loop {
        esp_println::println!("Spawn Task Count: {}", count);
        count += 1;
        Timer::after(Duration::from_millis(1_000)).await;
        match pin.output_level() {
            Level::High => pin.set_low(),
            Level::Low => pin.set_high(),
        }
    }
}

#[main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    // let io = Io::new(peripherals.IO_MUX);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    embassy::init(timg0.timer0);

    let ledpin: Output<'static> = Output::new(peripherals.GPIO18, Level::Low);

    spawner.spawn(one_second_task(ledpin)).unwrap();

    // This line is for Wokwi only so that the console output is formatted correctly
    print!("\x1b[20h");

    let mut count = 0;
    loop {
        esp_println::println!("Main Task Count: {}", count);
        count += 1;
        Timer::after(Duration::from_millis(5_000)).await;
    }
}
