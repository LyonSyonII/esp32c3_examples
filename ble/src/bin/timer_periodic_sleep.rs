#![no_std]
#![no_main]

use defmt::{info, warn};
use esp_hal::{
    gpio::Level,
    macros::{handler, ram},
    peripherals::Peripherals,
    rtc_cntl::{sleep::WakeSource, Rtc, RtcClock},
    timer::systimer::{self, SystemTimer},
    InterruptConfigurable,
};

esp32c3::setup_entry!(main);

fn main(peripherals: Peripherals) -> ! {
    // let systimer = systimer::SystemTimer::new(peripherals.SYSTIMER).split();
    /*     let mut alarm0 = systimer.alarm0.into_periodic();
    alarm0.set_period(fugit::MicrosDurationU32::from_ticks((SystemTimer::ticks_per_second() * 5) as u32));
    alarm0.set_interrupt_handler(alarm0_handler); */

    let timer_wakeup_source =
        esp_hal::rtc_cntl::sleep::TimerWakeupSource::new(core::time::Duration::from_secs(5));
    let mut sleep = Rtc::new(peripherals.LPWR);

    sleep.rwdt.disable();
    sleep.swd.disable();
    let mut pin = esp_hal::gpio::Output::new(peripherals.GPIO2, Level::High);
    loop {
        sleep.sleep_light(&[&timer_wakeup_source]);
        pin.toggle();
    }
}

#[handler]
#[ram]
fn alarm0_handler() {
    warn!("BEEP BEEP");
}
