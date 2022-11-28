//! i.MX RT target support.
//!
//! Defines a `cortex-m-rt` pre-init function that disables watchdogs and initializes TCM.
//! It then copies instructions, read-only data, and the vector table to their intended location.
//! This only happens if LMAs and VMAs differ.
//!
//! There's a few behaviors worth mentioning:
//!
//! - The implementation never clears the INIT_xTCM_EN bits in GPR16 if the xTCM is unused.
//!   The first reason is because we can't do this on the 1170 chips; the bits are reserved and
//!   should always be set (guessing its for the CM4, which always uses TCM). The second reason
//!   is that it doesn't seem necessary; we'll let xTCM initialize out of non-POR reset. From what
//!   I could gather, this would be the case if we set fuse values to specify an all-OCRAM config,
//!   and nothing says we need to flip these bits if the _fuses_ don't allocate xTCM. (Maybe this
//!   automagically happens? Not sure.)
//! - We're not changing CM7_xTCMSZ to reflect the xTCM sizes. Again, the setting isn't available
//!   on the 1170 chips. It's also OK to keep the POR value, since it represents the maximum-possible
//!   TCM size. This means that users have finer control over xTCM memory sizes, but invalid xTCM accesses
//!   won't cause a bus fault. See 3.1.3.2. in AN12077 for more discussion.
//!
//! Other notes:
//!
//! It's important that something sets the stack pointer. On the 10xx, the boot ROM sets the stack
//! pointer. But on the 11xx, the boot ROM doesn't set the stack pointer. See the link below for
//! more information. This implementation relies on the cortex-m-rt 0.7.2 "set-sp" feature to always
//! set the stack pointer, no matter the target chip.
//!
//! <https://community.nxp.com/t5/i-MX-RT/RT1176-ROM-code-does-not-set-stack-pointer-correctly/td-p/1388830>

use core::{arch::global_asm, ffi::c_void};

pub use cortex_m_rt::*;

global_asm! {r#"
.cfi_sections .debug_frame
.section .__pre_init,"ax"
.global __pre_init
.type __pre_init,%function
.thumb_func
.cfi_startproc

__pre_init:
    ldr r0, =__imxrt_family         @ Need to know which chip family we're initializing.
    ldr r1, =1170
    cmp r0, r1                      @ Is this an 1170?

    # Disable RTWODOG3.
    ite eq
    ldreq r2, =0x40038000           @ RTWDOG base address for 11xx chips...
    ldrne r2, =0x400BC000           @ RTWDOG base address for 10xx chips...
    ldr r3, =0xD928C520             @ RTWDOG magic number
    str r3, [r2, #4]                @ RTWDOG[CNT] = 0xD928C520.
    ldr r3, [r2]                    @ r3 = RTWDOG[CS]
    bic r3, r3, #1<<7               @ r3 = r3 & !(1 << 7), clears enable.
    str r3, [r2]                    @ RTWDOG[CS] = r3

    # Prepare FlexRAM regions.
    ldr r0, =0x400AC000             @ IMXRT_IOMUXC_GPR base address for 10xx chips, overwritten if actually 11xx...
    ldr r1, =__flexram_config       @ Value for GPR17 (and GPR18 for 11xx)
    itttt eq                        @ Need a few extra operations to handle 1170 split banks.
    ldreq r0, =0x400E4000           @ IMXRT_IOMUXC_GPR base address for 11xx chips, overwrite 10xx address...
    lsreq r2, r1, #16               @ r2 = ((unsigned)r1 >> 16)
    streq r2, [r0, #72]             @ *(IMXRT_IOMUXC_GPR + 18) = r2
    ubfxeq r1, r1, #0, #16          @ r1 = ((unsigned)r1 >> 0) & 0xFFFF, overwrite r1 with lower halfword.
    str r1, [r0, #68]               @ *(IMXRT_IOMUXC_GPR + 17) = r1
    ldr r1, [r0, #64]               @ r1 = *(IMXRT_IOMUXC_GPR + 16)
    orr r1, r1, #1<<2               @ r1 |= 1 << 2
    str r1, [r0, #64]               @ *(IMXRT_IOMUXC_GPR + 16) = r1

    # Conditionally copy text.
    ldr r0, =__stext
    ldr r2, =__sitext
    cmp r2, r0
    beq 42f

    ldr r1, =__etext
    43:
    cmp r1, r0
    beq 42f
    ldm r2!, {{r3}}
    stm r0!, {{r3}}
    b 43b
    42:

    # Conditionally copy the vector table.
    ldr r0, =__svector_table
    ldr r2, =__sivector_table
    cmp r2, r0
    beq 52f

    ldr r1, =__evector_table
    53:
    cmp r1, r0
    beq 52f
    ldm r2!, {{r3}}
    stm r0!, {{r3}}
    b 53b
    52:

    # Conditionally copy read-only data.
    ldr r0, =__srodata
    ldr r2, =__sirodata
    cmp r2, r0
    beq 62f

    ldr r1, =__erodata
    63:
    cmp r1, r0
    beq 62f
    ldm r2!, {{r3}}
    stm r0!, {{r3}}
    b 63b
    62:

    # All done; back to the reset handler.
    bx lr

.cfi_endproc
.size __pre_init, . - __pre_init
"#
}

/// Returns a pointer to the end of the heap.
///
/// The returned pointer is guaranteed to be 4-byte aligned.
#[inline]
pub fn heap_end() -> *mut u32 {
    extern "C" {
        static mut __eheap: c_void;
    }
    unsafe { core::ptr::addr_of_mut!(__eheap) as _ }
}
