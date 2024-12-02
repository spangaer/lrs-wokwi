#![no_std]
#![no_main]

use embedded_hal_bus::spi::ExclusiveDevice;
use esp_hal::{delay::Delay, gpio::*, prelude::*, spi};

/* Display and graphics */
use mipidsi::{
    models::ILI9341Rgb565,
    options::{ColorOrder, Orientation, Rotation},
    Display,
};

use display_interface_spi::SPIInterface;

use names;

use embedded_graphics::{
    draw_target::DrawTarget, geometry::*, mono_font::MonoTextStyle, pixelcolor::*,
    prelude::RgbColor, text::Text, Drawable,
};

use embedded_hal;

use profont::PROFONT_24_POINT;

use esp_backtrace as _;
use esp_println::{print, println};
use fixes::enable_lf_nl_mode;

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
impl<T: ::embedded_hal::digital::InputPin<Error = core::convert::Infallible>> Button<T> {
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
            delay.delay_millis(30 as u32);
            self.check();
            if !self.button.is_low().unwrap() {
                Event::Pressed
            } else {
                Event::Nothing
            }
        } else if self.pressed && !pressed_now {
            delay.delay_millis(30 as u32);
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
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });

    enable_lf_nl_mode!();

    println!("About to initialize the SPI LED driver ILI9341");
    // let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    /* Name pins */
    // esp_hal pins
    let mosi = peripherals.GPIO7;
    let sck = peripherals.GPIO6;
    let miso = peripherals.GPIO9;
    // embedded_hal pin wrapping
    let cs = Output::new(peripherals.GPIO2, Level::High); // SPI device recommends high
    let rst = Output::new(peripherals.GPIO10, Level::Low);
    let dc = Output::new(peripherals.GPIO3, Level::Low);

    /* Configure SPI */
    // esp spi
    let spi = spi::master::Spi::new(peripherals.SPI2)
        .with_sck(sck)
        .with_mosi(mosi)
        .with_miso(miso);
    // clock needs to go to the device below ¯\_(ツ)_/¯
    // The clock rete configured here is how it's "communicated" on the bus it seems.
    // I guess this services a clock hint (it's written to a register).
    // In the end it's the device that drives the clock by flicking the pin,
    // so the actual rate will be "elastic".

    // embedded_hal SpiDevice
    // https://docs.esp-rs.org/esp-hal/esp-hal/0.21.1/esp32c3/esp_hal/spi/master/index.html#exclusive-access-to-the-spi-bus
    // delay is just a wait-loop, so creating one here has no cost
    let spiDevice = ExclusiveDevice::new(spi, cs, Delay::new()).unwrap();

    let di = SPIInterface::new(spiDevice, dc);

    let mut delay = Delay::new();
    let mut display = mipidsi::Builder::new(ILI9341Rgb565, di)
        .display_size(240 as u16, 320 as u16)
        .orientation(Orientation::new().rotate(Rotation::Deg270))
        .color_order(ColorOrder::Bgr)
        .reset_pin(rst)
        .init(&mut delay)
        .unwrap();

    println!("Initialized");

    fn start<DI: display_interface::WriteOnlyDataCommand, RST: embedded_hal::digital::OutputPin>(
        text_box: Point,
        display: &mut Display<DI, mipidsi::models::ILI9341Rgb565, RST>,
    ) {
        let _ = display.clear(Rgb565::WHITE);

        Text::new(
            "Start",
            text_box,
            MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::BLACK),
        )
        .draw(display)
        .unwrap();
    }

    let text_box = display.bounding_box().center() - Size::new(80, 0);

    let mut button_yellow = Button::new(Input::new(peripherals.GPIO0, Pull::Up));
    let mut button_purple = Button::new(Input::new(peripherals.GPIO1, Pull::Up));
    let mut button_green = Button::new(Input::new(peripherals.GPIO8, Pull::Up));

    start(text_box, &mut display);

    loop {
        if let Event::Pressed = button_yellow.poll(&mut delay) {
            println!("pressed yellow");
            let _ = display.clear(Rgb565::CSS_AZURE);

            Text::new(
                names::ms_yellow(),
                text_box,
                MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::CSS_DARK_GOLDENROD),
            )
            .draw(&mut display)
            .unwrap();
        }

        if let Event::Pressed = button_purple.poll(&mut delay) {
            println!("pressed purple");
            let _ = display.clear(Rgb565::CSS_MOCCASIN);

            Text::new(
                names::mr_purple(),
                text_box,
                MonoTextStyle::new(&PROFONT_24_POINT, Rgb565::CSS_PURPLE),
            )
            .draw(&mut display)
            .unwrap();
        }

        if let Event::Pressed = button_green.poll(&mut delay) {
            println!("pressed green");
            start(text_box, &mut display);
        }
    }
}
