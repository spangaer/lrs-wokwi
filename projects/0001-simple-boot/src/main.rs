#![no_std]
#![no_main]
use esp_backtrace as _;
use esp_hal::{clock::CpuClock, delay::Delay, prelude::entry};
use esp_println::print; // used by println fixed macro
use fixes::println;

#[entry]
fn main() -> ! {
    let _peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    let delay = Delay::new();

    delay.delay_millis(1000);

    println!("Hello, world!");

    let mut i = 0_u32;

    loop {
        i += 1;
        println!("Hello {:?}", i);
        delay.delay_millis(1000);
    }
}
