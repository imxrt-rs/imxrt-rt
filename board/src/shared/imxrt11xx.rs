//! Code shared across all i.MX RT 11xx chips.

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
            __INTERRUPTS[crate::ral::Interrupt::PIT1 as usize]
                .get()
                .write_volatile(PIT);
        }
    }

    let ccm = unsafe { ral::ccm::CCM::instance() };

    // Change the bus clock to the 24 MHz XTAL.
    // Wouldn't recommend doing this in a real system,
    // since the bus clock is running rather slowly.
    // But, it's good enough for a demo, and it lets us match
    // the behaviors of the 10xx examples.
    //
    // If we decrease the bus speed too much, we seem to reach a condition
    // where we can't re-flash the device. Haven't dug too deeply; only
    // observed that keeping the bus clock faster lets us flash more reliably,
    // at least with pyOCD.
    let clock_root_2 = &ccm.CLOCK_ROOT[2];
    ral::modify_reg!(ral::ccm::clockroot, clock_root_2, CLOCK_ROOT_CONTROL, MUX: 0b001, DIV: 0);
    while ral::read_reg!(
        ral::ccm::clockroot,
        clock_root_2,
        CLOCK_ROOT_STATUS0,
        CHANGING == 1
    ) {}

    // Enable the clock gate to PIT1.
    ral::write_reg!(ral::ccm, ccm, LPCG61_DIRECT, 1);

    let pit = unsafe { ral::pit::PIT1::instance() };
    // 24x scaling accounts for the 24x faster PIT clock.
    // Looks like it's blinking at 1Hz, but I'm not pulling out
    // my scope or stopwatch or anything.
    Some(crate::Pit::new(
        &pit,
        timer_delay_microseconds.saturating_mul(24),
    ))
}
