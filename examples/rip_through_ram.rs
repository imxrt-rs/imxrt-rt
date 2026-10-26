//! Read through all of ITCM, DTCM, and OCRAM.
//!
//! Useful for stress testing ECC RAM. This program should complete
//! without activating any ECC RAM faults, provided the runtime has
//! preloaded all of RAM.
//!
//! This program assumes the layout specified in board/build.rs for
//! the 1170EVK.

#![no_std]
#![no_main]

const ITCM: (usize, usize) = (0x0000_0000, 0x0000_0000 + 256 * 1024);
const DTCM: (usize, usize) = (0x2000_0000, 0x2000_0000 + 256 * 1024);
const OCRAM: (usize, usize) = (0x2024_0000, 0x2024_0000 + 1024 * 1024);

// Single- and multi-bit errors in all three regions.
const FLEXRAM_STATUS_MASK: u32 = 0x3FC0;
// Single- and multi-bit errors in all banks.
const OCRAM_STATUS_MASK: u32 = 0xFFFF;

#[rtic::app(device = board::rtic_support, peripherals = false)]
mod app {
    use board::ral;

    #[local]
    struct Local {
        led: board::Led,
        flexram: ral::flexram::FLEXRAM,
        mecc1: ral::mecc::MECC1,
        mecc2: ral::mecc::MECC2,
    }

    #[shared]
    struct Shared {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        let board::Resources { led, .. } = board::prepare(1_000_000).unwrap();
        let flexram = unsafe { ral::flexram::FLEXRAM::instance() };
        let mecc1 = unsafe { ral::mecc::MECC1::instance() };
        let mecc2 = unsafe { ral::mecc::MECC2::instance() };

        ral::write_reg!(
            ral::flexram,
            flexram,
            INT_STAT_EN,
            crate::FLEXRAM_STATUS_MASK
        );
        ral::write_reg!(
            ral::flexram,
            flexram,
            INT_SIG_EN,
            crate::FLEXRAM_STATUS_MASK
        );

        ral::write_reg!(ral::mecc, mecc1, ERR_STAT_EN, crate::OCRAM_STATUS_MASK);
        ral::write_reg!(ral::mecc, mecc2, ERR_STAT_EN, crate::OCRAM_STATUS_MASK);
        ral::write_reg!(ral::mecc, mecc1, ERR_SIG_EN, crate::OCRAM_STATUS_MASK);
        ral::write_reg!(ral::mecc, mecc2, ERR_SIG_EN, crate::OCRAM_STATUS_MASK);

        (
            Shared {},
            Local {
                led,
                flexram,
                mecc1,
                mecc2,
            },
            init::Monotonics(),
        )
    }

    #[idle(local = [led, flexram, mecc1, mecc2])]
    fn idle(cx: idle::Context) -> ! {
        rtt_target::rprintln!("Starting to read all of RAM...");
        for (start, end) in [crate::ITCM, crate::DTCM, crate::OCRAM] {
            unsafe { crate::read_ram_range(start as *const u32, end as *const u32) };
            rtt_target::rprintln!("Completed range {:#010X} through {:#010X}", start, end);
        }

        let int_status = ral::read_reg!(ral::flexram, cx.local.flexram, INT_STATUS);
        assert_eq!(int_status, 0, "FlexRAM INT_STATUS: {int_status:#010X}");

        let err_status_1 = ral::read_reg!(ral::mecc, cx.local.mecc1, ERR_STATUS);
        let err_status_2 = ral::read_reg!(ral::mecc, cx.local.mecc2, ERR_STATUS);

        assert_eq!(err_status_1, 0, "MECC1 ERR_STATUS: {err_status_1:#010X}");
        assert_eq!(err_status_2, 0, "MECC2 ERR_STATUS: {err_status_2:#010X}");

        rtt_target::rprintln!("OK");
        loop {
            cx.local.led.set();
        }
    }

    // Expected to be linked by today's board...
    #[task(binds = PIT)]
    fn pit(_: pit::Context) {
        panic!("Unexpected PIT interrupt");
    }

    #[task(binds = FLEXRAM)]
    fn flexram(_: flexram::Context) {
        crate::flexram_report(&unsafe { ral::flexram::FLEXRAM::instance() });
        panic!("FLEXRAM");
    }

    #[task(binds = FLEXRAM_ECC)]
    fn flexram_ecc(_: flexram_ecc::Context) {
        crate::flexram_report(&unsafe { ral::flexram::FLEXRAM::instance() });
        panic!("FLEXRAM_ECC");
    }

    #[task(binds = MECC1_INT)]
    fn mecc1(_: mecc1::Context) {
        crate::mecc64_report(&unsafe { ral::mecc::MECC1::instance() });
        panic!("MECC1");
    }

    #[task(binds = MECC1_FATAL_INT)]
    fn mecc1_fatal(_: mecc1_fatal::Context) {
        crate::mecc64_report(&unsafe { ral::mecc::MECC1::instance() });
        panic!("MECC1_FATAL");
    }

    #[task(binds = MECC2_INT)]
    fn mecc2(_: mecc2::Context) {
        crate::mecc64_report(&unsafe { ral::mecc::MECC2::instance() });
        panic!("MECC2");
    }

    #[task(binds = MECC2_FATAL_INT)]
    fn mecc2_fatal(_: mecc2_fatal::Context) {
        crate::mecc64_report(&unsafe { ral::mecc::MECC2::instance() });
        panic!("MECC2_FATAL");
    }
}

use board::ral;

fn flexram_report(flexram: &ral::flexram::FLEXRAM) {
    macro_rules! print_regs {
        ($($reg:ident),* $(,)?) => {
            $(
                let value = ral::read_reg!(ral::flexram, flexram, $reg);
                rtt_target::rprintln!("FlexRAM {}: {:#010X}", stringify!($reg), value);
            )*
        };
    }

    print_regs!(
        INT_STATUS,
        ITCM_ECC_SINGLE_ERROR_INFO,
        ITCM_ECC_SINGLE_ERROR_ADDR,
        ITCM_ECC_MULTI_ERROR_INFO,
        ITCM_ECC_MULTI_ERROR_ADDR,
        D0TCM_ECC_SINGLE_ERROR_INFO,
        D0TCM_ECC_SINGLE_ERROR_ADDR,
        D0TCM_ECC_MULTI_ERROR_INFO,
        D0TCM_ECC_MULTI_ERROR_ADDR,
        D1TCM_ECC_SINGLE_ERROR_INFO,
        D1TCM_ECC_SINGLE_ERROR_ADDR,
        D1TCM_ECC_MULTI_ERROR_INFO,
        D1TCM_ECC_MULTI_ERROR_ADDR,
    );
}

fn mecc64_report<const N: u8>(mecc: &ral::mecc::Instance<N>) {
    macro_rules! print_regs {
        ($($reg:ident),* $(,)?) => {
            $(
                let value = ral::read_reg!(ral::mecc, mecc, $reg);
                rtt_target::rprintln!("MECC{} {}: {:#010X}", N, stringify!($reg), value);
            )*
        };
    }

    print_regs!(
        ERR_STATUS,
        SINGLE_ERR_ADDR_ECC0,
        SINGLE_ERR_ADDR_ECC1,
        SINGLE_ERR_ADDR_ECC2,
        SINGLE_ERR_ADDR_ECC3,
        MULTI_ERR_ADDR_ECC0,
        MULTI_ERR_ADDR_ECC1,
        MULTI_ERR_ADDR_ECC2,
        MULTI_ERR_ADDR_ECC3,
    );
}

#[unsafe(naked)]
unsafe extern "C" fn read_ram_range(start: *const u32, end: *const u32) {
    core::arch::naked_asm!(
        "1:",
        "cmp r0, r1",
        "beq 2f",
        "ldrd r2, r3, [r0], #8",
        "b 1b",
        "2:",
        "bx lr",
    )
}

use imxrt_rt::exception;

#[exception]
unsafe fn DefaultHandler(i: i16) {
    crate::flexram_report(&unsafe { ral::flexram::FLEXRAM::instance() });
    panic!("DefaultHandler {i}");
}
