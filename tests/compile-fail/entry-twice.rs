#![no_main]
#![no_std]

extern crate imxrt_rt;
extern crate panic_halt;

use imxrt_rt::entry;

#[entry]
fn foo() -> ! {
    loop {}
}

#[entry] //~ ERROR symbol `main` is already defined
fn bar() -> ! {
    loop {}
}
