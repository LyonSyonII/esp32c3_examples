//! BLE Example
//!
//! - starts Bluetooth advertising
//! - offers one service with three characteristics (one is read/write, one is write only, one is read/write/notify)
//! - pressing the boot-button on a dev-board will send a notification if it is subscribed

//% FEATURES: esp-wifi esp-wifi/ble
//% CHIPS: esp32 esp32s3 esp32c2 esp32c3 esp32c6 esp32h2

#![no_std]
#![no_main]

use bleps::{
    ad_structure::{
        create_advertising_data,
        AdStructure,
        BR_EDR_NOT_SUPPORTED,
        LE_GENERAL_DISCOVERABLE,
    },
    attribute_server::{AttributeServer, NotificationData, WorkResult},
    gatt,
    Ble,
    HciConnector,
};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    gpio::{Input, Pull},
    prelude::*,
    rng::Rng,
    timer::timg::TimerGroup,
};
use esp_wifi::{ble::controller::BleConnector, init};

use defmt_rtt as _;
use defmt::{error, info, warn};
use heapless::String;

#[entry]
fn main() -> ! {
    // SAFETY: Workaround for rust-analyzer to correctly see esp32c3::Peripherals, they are the same type
    #[allow(clippy::useless_transmute)]
    let peripherals: esp_hal::peripherals::Peripherals = unsafe { core::mem::transmute(esp_hal::init(esp_hal::Config::default())) };
    
    esp_alloc::heap_allocator!(72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let init = init(
        timg0.timer0,
        Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    let mut bluetooth = peripherals.BT;
    loop {
        start_ble(&init, &mut bluetooth);
    }
}

fn start_ble(init: &esp_wifi::EspWifiController<'_>, bluetooth: &mut impl esp_hal::peripheral::Peripheral<P = esp_hal::peripherals::BT>) {
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
        .unwrap()
    ).unwrap();
    ble.cmd_set_le_advertise_enable(true).unwrap();
    
    info!("BLE: started advertising");
    
    const LEN: usize = 300;
    let name = core::cell::RefCell::<heapless::String::<LEN>>::new(heapless::String::new());
    
    let mut rf = |offset: usize, data: &mut [u8]| {
        use core::fmt::Write as _;
        
        let name = name.borrow();
        let mut s = String::<LEN>::new();
        if name.is_empty() {
            write!(&mut s, "Hello, send your name please 123456789 123456789 123456789 123456789").unwrap();
        } else {
            write!(&mut s, "Hello {name}!").unwrap();
        }

        let s = &s[offset..];
        if offset == 0 {
            info!("## START OF MESSAGE ##");
            info!("MSG: {} bytes\tTXT: {:?}", s.len(), s);
        } else {
            info!("SNT: {} bytes\tREM: {:?}", offset, s.len());
        }
        data[..s.len()].copy_from_slice(s.as_bytes());
        
        s.len()
    };
    let mut wf = |offset: usize, data: &[u8]| {
        let Ok(v) = heapless::Vec::<u8, LEN>::from_slice(data) else {
            return warn!("RECEIVED DATA WITH LENGTH {}", data.len());
        };
        let Ok(s) = heapless::String::from_utf8(v) else {
            return warn!("RECEIVED DATA IS NOT UTF8");
        };
        let mut name = name.borrow_mut();
        name.clear();
        name.push_str(&s).unwrap();

        info!("RECEIVED NAME: {} {:?}", offset, s);
    };
    
    gatt!([service {
        uuid: "937312e0-2354-11eb-9f10-fbc30a62cf38",
        characteristics: [
            characteristic {
                name: "readwrite",
                uuid: "937312e0-2354-11eb-9f10-fbc30a62cf38",
                read: rf,
                write: wf,
            }
        ]
    }]);

    let mut rng = bleps::no_rng::NoRng;
    let mut srv = AttributeServer::new(&mut ble, &mut gatt_attributes, &mut rng);
    
    loop {
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