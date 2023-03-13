//! iMXRT1010EVK support.

use crate::ral;

#[cfg(all(target_arch = "arm", not(feature = "imxrt1010evk-ram")))]
use imxrt1010evk_fcb as _;
#[cfg(target_arch = "arm")]
use panic_rtt_target as _;

const LED_OFFSET: u32 = 11;

pub mod rtic_support {
    pub use crate::ral::*;
}

/// Prepare the board for the examples.
///
/// Call this first. Panics if something went wrong.
pub fn prepare(timer_delay_microseconds: u32) -> Option<crate::Resources> {
    #[cfg(target_arch = "arm")]
    rtt_target::rtt_init_print!();

    let iomuxc = unsafe { ral::iomuxc::IOMUXC::instance() };
    // Set the GPIO pad to a GPIO function (ALT 5)
    ral::write_reg!(ral::iomuxc, iomuxc, SW_MUX_CTL_PAD_GPIO_11, 5);
    // Increase drive strength, but leave other fields at their current value...
    ral::modify_reg!(ral::iomuxc, iomuxc, SW_PAD_CTL_PAD_GPIO_11, DSE: DSE_7_R0_7);

    let pit = crate::prepare_pit(timer_delay_microseconds)?;

    let gpio1 = unsafe { ral::gpio::GPIO1::instance() };
    Some(crate::Resources {
        led: crate::Led::new(LED_OFFSET, &gpio1),
        pit,
    })
}
