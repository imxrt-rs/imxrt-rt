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
fn SysTick(undef: u32) {}
//~^ ERROR `#[exception]` handlers other than `DefaultHandler` and `HardFault` must have signature `[unsafe] fn() [-> !]`
