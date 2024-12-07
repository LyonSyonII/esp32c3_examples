#![no_std]

use core::cell::{OnceCell, RefCell};

use critical_section::{CriticalSection, Mutex};
use esp_hal::{
    interrupt::InterruptHandler,
    timer::{
        systimer::{self, SystemTimer},
        Timer,
    },
};

/// Sets up [`esp_alloc`], [`esp_backtrace`] and [`defmt_rtt`].
///
/// Takes the [`Peripherals`](::esp_hal::peripherals::Peripherals) and calls the provided function.
///
/// An optional [`esp_hal::Config`] can be provided to change the default init configuration.
///
/// # Example
/// ```
/// use esp_hal::peripherals::Peripherals;
///
/// esp32c3::setup_entry!(main);
///
/// fn main(peripherals: Peripherals) -> ! {
///     // do someting...
/// }
/// ```
#[macro_export]
macro_rules! setup_entry {
    ($entrypoint:ident, $config:expr) => {
        use ::defmt_rtt as _;
        use ::esp_alloc as _;
        use ::esp_backtrace as _;

        #[::esp_hal::entry]
        fn entry() -> ! {
            // SAFETY: Workaround for rust-analyzer to correctly see esp32c3::Peripherals, they are the same type
            #[allow(clippy::useless_transmute)]
            let peripherals: ::esp_hal::peripherals::Peripherals =
                unsafe { ::core::mem::transmute(::esp_hal::init($config)) };

            ::esp_alloc::heap_allocator!(72 * 1024);

            let entrypoint: fn(::esp_hal::peripherals::Peripherals) -> ! = $entrypoint;
            entrypoint(peripherals)
        }
    };
    ($entrypoint:ident) => {
        $crate::setup_entry!($entrypoint, ::esp_hal::Config::default());
    };
}

#[inline(always)]
pub fn init() -> esp_hal::peripherals::Peripherals {
    // SAFETY: Workaround for rust-analyzer to correctly see esp32c3::Peripherals, they are the same type
    #[allow(clippy::useless_transmute)]
    let peripherals: esp_hal::peripherals::Peripherals =
        unsafe { core::mem::transmute(esp_hal::init(esp_hal::Config::default())) };

    esp_alloc::heap_allocator!(72 * 1024);

    peripherals
}
