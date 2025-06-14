//! Code shared across all i.MX RT 10xx chips.
use crate::ral;

pub(crate) fn prepare_pit(timer_delay_microseconds: u32) -> Option<crate::Pit> {
    #[cfg(feature = "rtic")]
    {
        unsafe extern "C" {
            // Not actually mut in cortex-m. But, no one is reading it...
            static __INTERRUPTS: [core::cell::UnsafeCell<unsafe extern "C" fn()>; 240];
            fn PIT();
        }
        unsafe {
            __INTERRUPTS[crate::ral::interrupt::PIT as usize]
                .get()
                .write_volatile(PIT);
        }
    }
    let ccm = unsafe { ral::ccm::CCM::instance() };
    // Disable the PIT clock gate while we change the clock...
    ral::modify_reg!(ral::ccm, ccm, CCGR1, CG6: 0b00);
    // Set the periodic clock divider, selection.
    // 24MHz crystal oscillator, divided by 24 == 1MHz PIT clock
    ral::modify_reg!(
        ral::ccm,
        ccm,
        CSCMR1,
        PERCLK_PODF: DIVIDE_24,
        PERCLK_CLK_SEL: PERCLK_CLK_SEL_1 // Oscillator clock
    );
    // Re-enable PIT clock
    ral::modify_reg!(ral::ccm, ccm, CCGR1, CG6: 0b11);

    let pit = unsafe { ral::pit::PIT::instance() };
    Some(crate::Pit::new(&pit, timer_delay_microseconds))
}
