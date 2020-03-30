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
fn HardFault(_ef: &ExceptionFrame, undef: u32) -> ! {
    //~^ ERROR `HardFault` handler must have signature `[unsafe] fn(&ExceptionFrame) -> !`
    loop {}
}
