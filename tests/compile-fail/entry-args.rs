#![no_main]
#![no_std]

extern crate imxrt_rt;
extern crate panic_halt;

use imxrt_rt::entry;

#[entry(foo)] //~ ERROR This attribute accepts no arguments
fn foo() -> ! {
    loop {}
}
