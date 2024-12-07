//! Demonstrates deep sleep with timer wakeup

//% CHIPS: esp32 esp32c3 esp32c6 esp32s3 esp32c2

#![no_std]
#![no_main]

use core::{cell::RefCell, fmt::Debug, time::Duration};

use critical_section::Mutex;
use defmt::info;
use esp_backtrace as _;
use esp_hal::prelude::*;
use esp_hal::{
    delay::Delay,
    entry,
    gpio::{Input, Io},
    macros::{handler, ram},
    peripherals::Peripherals,
    rtc_cntl::{reset_reason, sleep::TimerWakeupSource, wakeup_cause, Rtc, SocResetReason},
    Cpu,
};

static BUTTON: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));

esp32c3::setup_entry!(main);

fn main(peripherals: esp_hal::peripherals::Peripherals) -> ! {
    let mut io = Io::new(peripherals.IO_MUX);
    io.set_interrupt_handler(handler);
    let mut button = Input::new(peripherals.GPIO9, esp_hal::gpio::Pull::Up);
    critical_section::with(|cs| {
        button.listen(esp_hal::gpio::Event::FallingEdge);
        BUTTON.borrow_ref_mut(cs).replace(button);
    });

    let delay = Delay::new();
    loop {
        critical_section::with(|cs| {
            let button = BUTTON.borrow_ref(cs);
            info!("{}", button.as_ref().unwrap().is_low());
        });
        delay.delay_millis(50);
    }
}

#[handler]
#[ram]
fn handler() {
    info!("GPIO Interrupt");

    critical_section::with(|cs| {
        let mut cell = BUTTON.borrow_ref_mut(cs);
        let button = cell.as_mut().unwrap();
        if button.is_interrupt_set() {
            button.clear_interrupt();
            true
        } else {
            false
        }
    })
    .then(|| {
        info!("Button was the source of the interrupt");
    });
}
