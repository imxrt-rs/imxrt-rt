#![no_main]
#![no_std]

extern crate imxrt_rt;
extern crate panic_halt;

use imxrt_rt::{entry, exception};

#[entry]
fn foo() -> ! {
    loop {}
}

#[exception]
fn SysTick() {}

pub mod reachable {
    use imxrt_rt::exception;

    #[exception] //~ ERROR symbol `SysTick` is already defined
    fn SysTick() {}
}
