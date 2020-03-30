#![no_main]
#![no_std]

extern crate imxrt_rt;
extern crate panic_halt;

use imxrt_rt::{entry, pre_init};

#[pre_init(foo)] //~ ERROR This attribute accepts no arguments
unsafe fn foo() {}

#[entry]
fn baz() -> ! {
    loop {}
}
