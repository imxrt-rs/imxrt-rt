#![no_main]
#![no_std]

extern crate imxrt_rt;
extern crate panic_halt;

use imxrt_rt::{entry, exception, ExceptionFrame};

#[entry]
fn foo() -> ! {
    loop {}
}

#[exception]
fn HardFault(_ef: &ExceptionFrame) -> ! {
    loop {}
}

pub mod reachable {
    use imxrt_rt::{exception, ExceptionFrame};

    #[exception] //~ ERROR symbol `HardFault` is already defined
    fn HardFault(_ef: &ExceptionFrame) -> ! {
        loop {}
    }
}
