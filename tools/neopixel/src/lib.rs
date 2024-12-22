#![no_std]

use core::iter;

use embassy_time::{Duration, Timer};
use esp_hal::rmt::{Error as RmtError, PulseCode, TxChannel};
use fugit::Duration as FDuration;

const PWM_PERIOD_NS: u32 = 1250;
const PWM_ZERO_HIGH_NS: u32 = 350;
const PWM_ZERO_LOW_NS: u32 = PWM_PERIOD_NS - PWM_ZERO_HIGH_NS;
const PWM_ONE_HIGH_NS: u32 = 900;
const PWM_ONE_LOW_NS: u32 = PWM_PERIOD_NS - PWM_ONE_HIGH_NS;
const PWM_RGB_BITS: u32 = 8 * 3;
const PWM_RGB_DATAFRAME: usize = PWM_RGB_BITS as usize + 1;
// const PWM_RGBSEND_US: u64 = (PWM_RGB_BITS * PWM_PERIOD_NS / 1000) as u64;
const NEOPX_LATCH_US: u64 = 55 as u64;

/// Note: passing the channel as a function param makes for clean borrow check handling
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

    fn encode_pixel(&self, color: Rgb) -> [u32; PWM_RGB_DATAFRAME] {
        let bits: u32 = color.into();

        let mut pixel_data = [self.pwm_zero; PWM_RGB_DATAFRAME];
        pixel_data[pixel_data.len() - 1] = PulseCode::empty(); // end marker

        for i in 0..PWM_RGB_BITS as usize {
            // Extract the i-th bit
            let bit = (bits >> (PWM_RGB_BITS as usize - 1 - i)) & 1;

            if bit > 0 {
                pixel_data[i] = self.pwm_one;
            }
        }

        pixel_data
    }

    pub async fn set_pixel<C: TxChannel>(
        &self,
        channel: C,
        color: Rgb,
    ) -> Result<C, (RmtError, C)> {
        self.set_pixels(channel, iter::once(color)).await
    }

    /// Will transmit to daisy chained Neopixels and let them latch
    pub async fn set_pixels<C: TxChannel>(
        &self,
        channel: C,
        data: impl Iterator<Item = Rgb>,
    ) -> Result<C, (RmtError, C)> {
        let mut out = channel;

        for pixel in data {
            let pixel_data = self.encode_pixel(pixel);
            out = out.transmit(&pixel_data).unwrap().wait()?;
        }

        Self::latch().await;
        Ok(out)
    }

    // /// Will transmit to daisy chained Neopixels and let them latch
    // ///
    // /// This implementation tried to parallelize the pixel calc with
    // /// rmt transmission, but that doesn't seem to fly in wokwi, the pixel simulation
    // /// seems to colide with the pixel simulation etc. So no form of yielding works.
    // /// The parrallel pixel calculation works the fisrt time around but not for next frames.
    // pub async fn set_pixels<C: TxChannel>(
    //     &self,
    //     channel: C,
    //     data: impl Iterator<Item = Rgb>,
    // ) -> Result<C, (RmtError, C)> {
    //     let mut out = Some(channel);
    //     let mut pixel_data;
    //     let mut waiter: Option<SingleShotTxTransaction<'_, C>> = None;

    //     for pixel in data {
    //         let new_data = self.encode_pixel(pixel);

    //         match waiter {
    //             None => (),
    //             Some(w) => {
    //                 out = Some(w.wait()?);
    //                 // waiter = None;
    //             }
    //         }

    //         pixel_data = new_data;

    //         waiter = Some(out.unwrap().transmit(&pixel_data).unwrap());
    //         out = None;
    //     }

    //     match waiter {
    //         None => (),
    //         Some(w) => {
    //             out = Some(w.wait()?);
    //             // waiter = None;
    //         }
    //     }

    //     Self::latch().await;
    //     Ok(out.unwrap())
    // }
}

#[derive(Debug, Clone, Copy)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    pub const BLACK: Rgb = Self { r: 0, g: 0, b: 0 };
    pub const RED: Rgb = Self { r: 255, g: 0, b: 0 };
    pub const GREEN: Rgb = Self { r: 0, g: 255, b: 0 };
    pub const BLUE: Rgb = Self { r: 0, g: 0, b: 255 };
    pub const YELLOW: Rgb = Self {
        r: 255,
        g: 255,
        b: 0,
    };
    pub const PURPLE: Rgb = Self {
        r: 120,
        g: 60,
        b: 255,
    };

    #[allow(unused)]
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<Rgb> for u32 {
    /// Convert RGB to u32 color value
    fn from(rgb: Rgb) -> Self {
        ((rgb.g as u32) << 16) | ((rgb.r as u32) << 8) | rgb.b as u32
    }
}
