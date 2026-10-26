//! Support for booting the Cortex M7 on the i.MX RT 1170 EVK.

use crate::ral;

#[cfg(all(target_arch = "arm", not(feature = "ram")))]
use imxrt1170evk_fcb as _;
#[cfg(target_arch = "arm")]
use panic_rtt_target as _;

const LED_OFFSET: u32 = 3;

pub mod rtic_support {
    pub use crate::ral::NVIC_PRIO_BITS;
    #[allow(non_snake_case)] // For RTIC trickery...
    pub mod Interrupt {
        pub const PIT: crate::ral::Interrupt = crate::ral::Interrupt::PIT1;
        pub const FLEXRAM: crate::ral::Interrupt = crate::ral::Interrupt::FLEXRAM;
        pub const FLEXRAM_ECC: crate::ral::Interrupt = crate::ral::Interrupt::FLEXRAM_ECC;
        pub const MECC1_INT: crate::ral::Interrupt = crate::ral::Interrupt::MECC1_INT;
        pub const MECC1_FATAL_INT: crate::ral::Interrupt = crate::ral::Interrupt::MECC1_FATAL_INT;
        pub const MECC2_INT: crate::ral::Interrupt = crate::ral::Interrupt::MECC2_INT;
        pub const MECC2_FATAL_INT: crate::ral::Interrupt = crate::ral::Interrupt::MECC2_FATAL_INT;
    }
    pub use Interrupt as interrupt;
}

pub fn prepare(timer_delay_microseconds: u32) -> Option<crate::Resources> {
    #[cfg(target_arch = "arm")]
    rtt_target::rtt_init_print!();

    let iomuxc = unsafe { ral::iomuxc::IOMUXC::instance() };
    ral::modify_reg!(ral::iomuxc, iomuxc, SW_MUX_CTL_PAD_GPIO_AD_04, MUX_MODE: 5);

    let ccm = unsafe { ral::ccm::CCM::instance() };
    // Enable LPCG for GPIOs.
    ral::write_reg!(ral::ccm, ccm, LPCG51_DIRECT, 1);

    let gpio = unsafe { ral::gpio::GPIO3::instance() };
    let pit = crate::prepare_pit(timer_delay_microseconds)?;
    Some(crate::Resources {
        pit,
        led: crate::Led::new(LED_OFFSET, &gpio),
    })
}
