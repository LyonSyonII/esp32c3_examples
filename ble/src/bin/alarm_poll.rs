#![no_std]
#![no_main]

use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    prelude::*,
    timer::systimer::{self, SystemTimer},
};

use defmt::info;
use defmt_rtt as _;

#[entry]
fn entry() -> ! {
    main()
}

fn main() -> ! {
    // SAFETY: Workaround for rust-analyzer to correctly see esp32c3::Peripherals, they are the same type
    #[allow(clippy::useless_transmute)]
    let peripherals: esp_hal::peripherals::Peripherals =
        unsafe { core::mem::transmute(esp_hal::init(esp_hal::Config::default())) };

    esp_alloc::heap_allocator!(72 * 1024);

    let systimer = systimer::SystemTimer::new(peripherals.SYSTIMER).split();
    let ticks_per_second = SystemTimer::ticks_per_second();
    let target_alarm = systimer.alarm0.into_target();

    let mut i = 1;
    target_alarm.set_target(SystemTimer::now() + ticks_per_second * i);
    target_alarm.enable_interrupt(true);

    let mut prev = 0;
    loop {
        let current = SystemTimer::now() / ticks_per_second;
        if current != prev {
            info!("[Time] {}", current);
            prev = current;
        }
        if target_alarm.is_interrupt_set() {
            info!("[Alarm] BEEP BEEP\n");
            i += 1;
            target_alarm.reset();
            target_alarm.set_target(SystemTimer::now() + ticks_per_second * i);
            target_alarm.clear_interrupt();
        }
    }
}
