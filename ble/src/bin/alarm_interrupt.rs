#![no_std]
#![no_main]

use core::{cell::{Cell, RefCell}, ops::AddAssign};

use critical_section::Mutex;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    prelude::*,
    timer::systimer::{self, Alarm, SystemTimer},
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
    
    target_alarm.set_target(SystemTimer::now() + ticks_per_second);
    target_alarm.set_interrupt_handler(alarm_handler);
    target_alarm.enable_interrupt(true);
    critical_section::with(|cs| ALARM0.replace(cs, Some(target_alarm)));

    let mut prev = 0;
    loop {
        let current = SystemTimer::now() / ticks_per_second;
        if current != prev {
            info!("[Time] {}", current);
            prev = current;
        }
    }
}

static SECONDS: Mutex<RefCell<u64>> = Mutex::new(RefCell::new(1));
static ALARM0: Mutex<RefCell<Option<Alarm<'_, systimer::Target, esp_hal::Blocking>>>> = Mutex::new(RefCell::new(None));

#[handler(priority = esp_hal::interrupt::Priority::max())]
#[ram]
fn alarm_handler() {
    info!("[Alarm] BEEP BEEP\n");
    critical_section::with(|cs| {
        let mut target_alarm = ALARM0.borrow_ref_mut(cs);
        let Some(target_alarm) = target_alarm.as_mut() else {
            return;
        };
        let mut seconds = SECONDS.borrow_ref_mut(cs);
        seconds.add_assign(1);
        target_alarm.reset();
        target_alarm.set_target(SystemTimer::now() + SystemTimer::ticks_per_second() * *seconds);
        target_alarm.clear_interrupt();
    });
}