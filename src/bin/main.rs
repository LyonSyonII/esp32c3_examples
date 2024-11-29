#![no_std]
#![no_main]

use esp_backtrace as _;
use esp_hal::gpio;
use esp_hal::{delay::Delay, prelude::*};

use defmt_rtt as _;
use defmt::{error, info, warn};

extern crate alloc;

#[entry]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    esp_alloc::heap_allocator!(72 * 1024);
    
    let mut led = gpio::Output::new(peripherals.GPIO0, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO1, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO2, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO3, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO4, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO5, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO6, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO7, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO8, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO9, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO10, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO11, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO12, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO13, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO14, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO15, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO16, gpio::Level::High);
    // led.set_high();
    // let mut led = gpio::Output::new(peripherals.GPIO17, gpio::Level::High);
    // led.set_high();
    let mut led = gpio::Output::new(peripherals.GPIO18, gpio::Level::High);
    led.set_high();
    
    let delay = Delay::new();
    loop {
        info!("Hello!");
        delay.delay(500.millis());
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/v0.22.0/examples/src/bin
}

#[handler]
#[ram]
fn io_interrupt_handler() {
    info!("GPIO Interrupt");
}