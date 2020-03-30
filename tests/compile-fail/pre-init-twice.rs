#![no_main]
#![no_std]

extern crate imxrt_rt;
extern crate panic_halt;

use imxrt_rt::{entry, pre_init};

#[pre_init]
unsafe fn foo() {}

#[pre_init] //~ ERROR symbol `__pre_init` is already defined
unsafe fn bar() {}

#[entry]
fn baz() -> ! {
    loop {}
}
