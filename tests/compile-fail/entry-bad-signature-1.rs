#![no_main]
#![no_std]

extern crate imxrt_rt;
extern crate panic_halt;

use imxrt_rt::entry;

#[entry]
fn foo() {}
//~^ ERROR `#[entry]` function must have signature `[unsafe] fn() -> !`