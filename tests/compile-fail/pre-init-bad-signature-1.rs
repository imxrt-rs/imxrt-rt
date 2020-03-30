#![no_main]
#![no_std]

extern crate imxrt_rt;
extern crate panic_halt;

use imxrt_rt::{entry, pre_init};

#[pre_init]
fn foo() {}
//~^ ERROR `#[pre_init]` function must have signature `unsafe fn()`

#[entry]
fn bar() -> ! {
    loop {}
}
