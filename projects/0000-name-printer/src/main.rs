#![no_std]
#![no_main]

use esp_hal::{
    clock::ClockControl, delay::Delay, gpio::InputPin, gpio::*, peripherals::Peripherals,
    prelude::*, spi, timer::TimerGroup, Rtc, IO,
};

/* Display and graphics */
use mipidsi::{
    options::{ColorOrder, Orientation},
    Display,
};

use display_interface_spi::SPIInterfaceNoCS;

use core::f32::consts::PI;
use libm::{cos, sin};

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::*,
    image::Image,
    mono_font::{ascii::FONT_10X20, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::*,
    prelude::RgbColor,
    prelude::*,
    primitives::{Circle, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle},
    text::*,
    text::{Alignment, Text},
    Drawable,
};

use embedded_hal;

use profont::{PROFONT_18_POINT, PROFONT_24_POINT};

use esp_backtrace as _;
use esp_println::println;

/* Debouncing algorythm */
pub enum Event {
    Pressed,
    Released,
    Nothing,
}
pub struct Button<T> {
    button: T,
    pressed: bool,
}
impl<T: ::embedded_hal::digital::v2::InputPin<Error = core::convert::Infallible>> Button<T> {
    pub fn new(button: T) -> Self {
        Button {
            button,
            pressed: true,
        }
    }
    pub fn check(&mut self) {
        self.pressed = !self.button.is_low().unwrap();
    }

    pub fn poll(&mut self, delay: &mut Delay) -> Event {
        let pressed_now = !self.button.is_low().unwrap();
        if !self.pressed && pressed_now {
            delay.delay_ms(30 as u32);
            self.check();
            if !self.button.is_low().unwrap() {
                Event::Pressed
            } else {
                Event::Nothing
            }
        } else if self.pressed && !pressed_now {
            delay.delay_ms(30 as u32);
            self.check();
            if self.button.is_low().unwrap() {
                Event::Released
            } else {
                Event::Nothing
            }
        } else {
            Event::Nothing
        }
    }
}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let mut system = peripherals.SYSTEM.split();
    let mut clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    println!("About to initialize the SPI LED driver ILI9341");
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    /* Set corresponding pins */
    let mosi = io.pins.gpio7;
    let cs = io.pins.gpio2;
    let rst = io.pins.gpio10;
    let dc = io.pins.gpio3;
    let sck = io.pins.gpio6;
    let miso = io.pins.gpio9;
    let backlight = io.pins.gpio4;

    /* Then set backlight (set_low() - display lights up when signal is in 0, set_high() - opposite case(for example.)) */
    let mut backlight = backlight.into_push_pull_output();
    //backlight.set_low().unwrap();

    /* Configure SPI */
    let spi = spi::master::Spi::new(
        peripherals.SPI2,
        100u32.MHz(),
        spi::SpiMode::Mode0,
        &mut clocks,
    )
    .with_pins(Some(sck), Some(mosi), Some(miso), Some(cs));

    let di = SPIInterfaceNoCS::new(spi, dc.into_push_pull_output());
    let reset = rst.into_push_pull_output();
    let mut delay = Delay::new(&clocks);

    let mut display: mipidsi::Display<
        SPIInterfaceNoCS<_, GpioPin<Output<PushPull>, 3>>,
        mipidsi::models::ILI9341Rgb565,
        GpioPin<Output<PushPull>, 10>,
    > = mipidsi::Builder::ili9341_rgb565(di)
        .with_display_size(240 as u16, 320 as u16)
        .with_framebuffer_size(240 as u16, 320 as u16)
        .with_orientation(Orientation::LandscapeInverted(true))
        .with_color_order(ColorOrder::Bgr)
        .init(&mut delay, Some(reset))
        .unwrap();

    println!("Initialized");

    let _ = display.clear(Rgb565::WHITE);

    let text_box = display.bounding_box().center() - Size::new(80, 0);

    let mut button_yellow = Button::new(io.pins.gpio0.into_pull_up_input());
    let mut button_blue = Button::new(io.pins.gpio1.into_pull_up_input());
    let mut button_green = Button::new(io.pins.gpio8.into_pull_up_input());

    // fn start(text_box: Point, display: Display<_, _, _>) {}

    Text::new(
        "Start",
        text_box,
        MonoTextStyle::new(&PROFONT_18_POINT, Rgb565::BLACK),
    )
    .draw(&mut display)
    .unwrap();

    loop {
        if let Event::Pressed = button_yellow.poll(&mut delay) {
            println!("pressed yellow");
            let _ = display.clear(Rgb565::WHITE);

            Text::new(
                "Yellow",
                text_box,
                MonoTextStyle::new(&PROFONT_18_POINT, Rgb565::CSS_DARK_GOLDENROD),
            )
            .draw(&mut display)
            .unwrap();
        }

        if let Event::Pressed = button_blue.poll(&mut delay) {
            println!("pressed blue");
            let _ = display.clear(Rgb565::WHITE);

            Text::new(
                "Blue",
                text_box,
                MonoTextStyle::new(&PROFONT_18_POINT, Rgb565::CSS_DARK_BLUE),
            )
            .draw(&mut display)
            .unwrap();
        }

        if let Event::Pressed = button_green.poll(&mut delay) {
            println!("pressed green");
            let _ = display.clear(Rgb565::WHITE);

            Text::new(
                "Start",
                text_box,
                MonoTextStyle::new(&PROFONT_18_POINT, Rgb565::BLACK),
            )
            .draw(&mut display)
            .unwrap();
        }
    }
}
