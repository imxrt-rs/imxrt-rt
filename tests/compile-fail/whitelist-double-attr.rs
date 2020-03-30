#![no_main]
#![no_std]

extern crate imxrt_rt;
extern crate panic_halt;

use imxrt_rt::{entry, exception};

#[exception]
#[entry] //~ ERROR this attribute is not allowed on an exception handler
fn SVCall() -> ! {
    loop {}
}
