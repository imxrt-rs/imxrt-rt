//! Rust entry point

#![no_std]

mod cache;
mod fpu;
mod nvic;

pub use cortex_m_rt_macros::{entry, exception, interrupt};
pub use nvic::exception;

use core::fmt;

/// Registers stacked (pushed into the stack) during an exception
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ExceptionFrame {
    /// (General purpose) Register 0
    pub r0: u32,

    /// (General purpose) Register 1
    pub r1: u32,

    /// (General purpose) Register 2
    pub r2: u32,

    /// (General purpose) Register 3
    pub r3: u32,

    /// (General purpose) Register 12
    pub r12: u32,

    /// Linker Register
    pub lr: u32,

    /// Program Counter
    pub pc: u32,

    /// Program Status Register
    pub xpsr: u32,
}

impl fmt::Debug for ExceptionFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        struct Hex(u32);
        impl fmt::Debug for Hex {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "0x{:08x}", self.0)
            }
        }
        f.debug_struct("ExceptionFrame")
            .field("r0", &Hex(self.r0))
            .field("r1", &Hex(self.r1))
            .field("r2", &Hex(self.r2))
            .field("r3", &Hex(self.r3))
            .field("r12", &Hex(self.r12))
            .field("lr", &Hex(self.lr))
            .field("pc", &Hex(self.pc))
            .field("xpsr", &Hex(self.xpsr))
            .finish()
    }
}

#[doc(hidden)]
#[link_section = ".HardFault.default"]
#[no_mangle]
pub unsafe extern "C" fn HardFault_(_: &ExceptionFrame) -> ! {
    loop {
        core::sync::atomic::spin_loop_hint();
    }
}


#[doc(hidden)]
#[no_mangle]
unsafe extern "C" fn DefaultHandler_() -> ! {
    loop {
        core::sync::atomic::spin_loop_hint();
    }
}


/// System entrypoint
///
/// # Safety
///
/// The function is unsafe since it directly modifies registers, and invokes
/// other functions that do the same.
#[no_mangle]
pub unsafe extern "C" fn _start() -> ! {
    nvic::init();
    fpu::init();
    cache::init();

    extern "Rust" {
        fn main() -> !;
    }

    #[inline(never)]
    fn trampoline() -> ! {
        unsafe { main() };
    }

    trampoline();
}
