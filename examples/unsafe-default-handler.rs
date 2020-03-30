#![deny(warnings)]
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
unsafe fn DefaultHandler(_irqn: i16) {}
