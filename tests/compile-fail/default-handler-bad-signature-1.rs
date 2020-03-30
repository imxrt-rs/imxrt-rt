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
fn DefaultHandler(_irqn: i16, undef: u32) {}
//~^ ERROR `DefaultHandler` must have signature `[unsafe] fn(i16) [-> !]`
