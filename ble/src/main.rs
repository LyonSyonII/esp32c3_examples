#![no_std]
#![no_main]

use core::{
    borrow::BorrowMut,
    cell::{Cell, RefCell},
};

use bleps::{
    ad_structure::{
        create_advertising_data, AdStructure, BR_EDR_NOT_SUPPORTED, LE_GENERAL_DISCOVERABLE,
    },
    attribute_server::{AttributeServer, NotificationData, WorkResult},
    gatt, Ble, HciConnector,
};
use critical_section::Mutex;
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, Pull},
    interrupt::InterruptHandler,
    prelude::*,
    rng::Rng,
    timer::{
        systimer::{self, Alarm, FrozenUnit, Periodic, SysTimerAlarms, SystemTimer, Target},
        timg::TimerGroup,
    },
};
use esp_wifi::{ble::controller::BleConnector, init};

use defmt::{error, info, trace, warn};
use defmt_rtt as _;
use heapless::String;

static ALARM0: critical_section::Mutex<
    RefCell<Option<(Alarm<'_, systimer::Target, esp_hal::Blocking>, u64)>>,
> = critical_section::Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // SAFETY: Workaround for rust-analyzer to correctly see esp32c3::Peripherals, they are the same type
    #[allow(clippy::useless_transmute)]
    let peripherals: esp_hal::peripherals::Peripherals =
        unsafe { core::mem::transmute(esp_hal::init(esp_hal::Config::default())) };

    esp_alloc::heap_allocator!(72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let init = init(
        timg0.timer0,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    let systimer = systimer::SystemTimer::new(peripherals.SYSTIMER).split();
    critical_section::with(|cs| {
        let alarm = systimer.alarm0.into_target(); // oneshot alarm
        alarm.enable_interrupt(false);
        alarm.set_interrupt_handler(alarm_handler);
        alarm.set_target(0);
        ALARM0.replace(cs, Some((alarm, 0)))
    });

    let mut bluetooth = peripherals.BT;
    loop {
        // ble_server(&init, &mut bluetooth);
    }
}

#[handler(priority = esp_hal::interrupt::Priority::max())]
#[ram]
fn alarm_handler() {
    critical_section::with(|cs| {
        info!("[Alarm Interrupt] BEEP BEEP BEEP");
        let mut alarm_cell = ALARM0.borrow_ref_mut(cs);
        let Some((alarm, _)) = alarm_cell.as_mut() else {
            return;
        };
        alarm.enable_interrupt(false);
        alarm.clear_interrupt();
    });
}

fn ble_receive_write(_offset: usize, data: &[u8]) {
    info!("[BLE-Write] Received data\t{}", data);
    if data.len() != 4 {
        error!("[BLE-Write] Received data with a len of {}", data.len());
        return;
    }

    let secs = u32::from_be_bytes(unsafe { data.try_into().unwrap_unchecked() }) as u64;
    critical_section::with(|cs| {
        info!("Inside critical");
        let mut alarm_cell = ALARM0.borrow_ref_mut(cs);
        let Some((alarm, target)) = alarm_cell.as_mut() else {
            return;
        };
        *target = SystemTimer::now() + secs * SystemTimer::ticks_per_second();
        alarm.set_target(*target);
        alarm.enable_interrupt(true);
        info!("Outside critical");
    });
    info!("[BLE-Write] Set alarm for \t{}\tseconds", secs);
}

fn ble_receive_read(_offset: usize, data: &mut [u8]) -> usize {
    let Some(mut remaining) = critical_section::with(|cs| {
        ALARM0
            .borrow_ref(cs)
            .as_ref()
            .map(|(alarm, target)| target.saturating_sub(alarm.now().ticks()))
    }) else {
        return 0;
    };
    remaining /= SystemTimer::ticks_per_second();

    info!("[BLE-Read] Remaining\t{}\tseconds", remaining);
    data[..4].copy_from_slice(&(remaining as u32).to_be_bytes());
    4
}

fn ble_server<'a>(
    init: &'a esp_wifi::EspWifiController<'_>,
    bluetooth: &'a mut impl esp_hal::peripheral::Peripheral<P = esp_hal::peripherals::BT>,
) {
    let now = || esp_hal::time::now().duration_since_epoch().to_millis();
    let connector = BleConnector::new(init, bluetooth);
    let hci = HciConnector::new(connector, now);
    let mut ble = Ble::new(&hci);

    ble.init().unwrap();
    ble.cmd_set_le_advertising_parameters().unwrap();
    ble.cmd_set_le_advertising_data(
        create_advertising_data(&[
            AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
            AdStructure::ServiceUuids16(&[Uuid::Uuid16(0x1809)]),
            AdStructure::CompleteLocalName(esp_hal::chip!()),
        ])
        .unwrap(),
    )
    .unwrap();
    ble.cmd_set_le_advertise_enable(true).unwrap();

    info!("BLE: started advertising");

    gatt!([service {
        uuid: "937312e0-2354-11eb-9f10-fbc30a62cf38",
        characteristics: [characteristic {
            name: "readwrite",
            uuid: "937312e0-2354-11eb-9f10-fbc30a62cf38",
            read: ble_receive_read,
            write: ble_receive_write,
        }]
    }]);

    let mut rng = bleps::no_rng::NoRng;
    let mut srv = AttributeServer::new(&mut ble, &mut gatt_attributes, &mut rng);

    loop {
        trace!("Calling main loop");

        match srv.do_work() {
            Ok(res) => {
                if let WorkResult::GotDisconnected = res {
                    break;
                }
            }
            Err(err) => {
                error!("{:?}", err);
            }
        }
    }
}
