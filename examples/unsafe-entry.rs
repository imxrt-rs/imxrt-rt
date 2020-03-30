#![deny(warnings)]
#![no_main]
#![no_std]

extern crate imxrt_rt;
extern crate panic_halt;

use imxrt_rt::entry;

#[entry]
unsafe fn foo() -> ! {
    loop {}
}
