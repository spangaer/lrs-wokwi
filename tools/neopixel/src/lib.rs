#![no_std]

use embassy_time::{Duration, Timer};
use esp_hal::rmt::{Error as RmtError, PulseCode, TxChannel};
use fugit::Duration as FDuration;

const PWM_PERIOD_NS: u32 = 1250;
const PWM_ZERO_HIGH_NS: u32 = 350;
const PWM_ZERO_LOW_NS: u32 = PWM_PERIOD_NS - PWM_ZERO_HIGH_NS;
const PWM_ONE_HIGH_NS: u32 = 900;
const PWM_ONE_LOW_NS: u32 = PWM_PERIOD_NS - PWM_ONE_HIGH_NS;
const PWM_RGB_BITS: u32 = 8 * 3;
const PWM_RGBSEND_US: u64 = (PWM_RGB_BITS * PWM_PERIOD_NS / 1000) as u64;
const NEOPX_LATCH_US: u64 = 55 as u64;

pub struct PixelSender<const RMT_TICK_RATE_HZ: u32> {
    pwm_zero: u32,
    pwm_one: u32,
}

impl<const RMT_TICK_RATE_HZ: u32> PixelSender<RMT_TICK_RATE_HZ> {
    pub fn new() -> PixelSender<RMT_TICK_RATE_HZ> {
        let pwm_zero = Self::pulse_from_intervals(PWM_ZERO_HIGH_NS, PWM_ZERO_LOW_NS);
        let pwm_one = Self::pulse_from_intervals(PWM_ONE_HIGH_NS, PWM_ONE_LOW_NS);

        PixelSender { pwm_zero, pwm_one }
    }

    fn pulse_from_intervals(nanos_high: u32, nanos_low: u32) -> u32 {
        let ticks_high = FDuration::<u32, 1, RMT_TICK_RATE_HZ>::nanos(nanos_high).ticks() as u16;
        let ticks_low = FDuration::<u32, 1, RMT_TICK_RATE_HZ>::nanos(nanos_low).ticks() as u16;
        <u32 as PulseCode>::new(true, ticks_high, false, ticks_low)
    }

    async fn latch() {
        Timer::after(Duration::from_micros(NEOPX_LATCH_US)).await;
    }

    async fn encode_and_send_pixel<C: TxChannel>(
        &self,
        channel: C,
        color: Rgb,
    ) -> Result<C, (RmtError, C)> {
        let bits: u32 = color.into();

        let mut pixel_data = [self.pwm_zero; PWM_RGB_BITS as usize + 1];
        pixel_data[pixel_data.len() - 1] = PulseCode::empty(); // end marker

        for i in 0..PWM_RGB_BITS as usize {
            let bit = (bits >> i) & 1; // Extract the i-th bit

            if bit > 0 {
                pixel_data[i] = self.pwm_one;
            }
        }

        // FIXME proper error handling
        let transaction = channel.transmit(&pixel_data).expect("transmit failed");

        Timer::after(Duration::from_micros(PWM_RGBSEND_US)).await;

        transaction.wait()
    }

    pub async fn set_pixel<C: TxChannel>(
        &self,
        channel: C,
        color: Rgb,
    ) -> Result<C, (RmtError, C)> {
        let out = self.encode_and_send_pixel(channel, color).await?;
        Self::latch().await;
        Ok(out)
    }

}

#[derive(Debug, Clone, Copy)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    pub const RED: Rgb = Self { r: 255, g: 0, b: 0 };
    pub const GREEN: Rgb = Self { r: 0, g: 255, b: 0 };
    pub const BLUE: Rgb = Self { r: 0, g: 0, b: 255 };

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
