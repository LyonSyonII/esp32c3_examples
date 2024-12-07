//! Demonstrates deep sleep with timer wakeup

//% CHIPS: esp32 esp32c3 esp32c6 esp32s3 esp32c2

#![no_std]
#![no_main]

use core::{fmt::Debug, time::Duration};

use defmt::info;
use esp_backtrace as _;
use esp_hal::{
    delay::Delay,
    entry,
    peripherals::Peripherals,
    rtc_cntl::{reset_reason, sleep::TimerWakeupSource, wakeup_cause, Rtc, SocResetReason},
    Cpu,
};

esp32c3::setup_entry!(main);

fn main(peripherals: Peripherals) -> ! {
    let delay = Delay::new();
    let mut rtc = Rtc::new(peripherals.LPWR);

    info!("up and runnning!");
    let reason = reset_reason(Cpu::ProCpu).unwrap_or(SocResetReason::ChipPowerOn);
    // info!("reset reason: {:?}", reason);
    let wake_reason = wakeup_cause();
    // info!("wake reason: {:?}", wake_reason);

    let timer = TimerWakeupSource::new(Duration::from_secs(5));
    info!("sleeping!");
    delay.delay_millis(100);
    rtc.sleep_deep(&[&timer]);
}
