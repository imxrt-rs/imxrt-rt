#![no_main]
#![no_std]

extern crate imxrt_rt;
extern crate panic_halt;

use imxrt_rt::{entry, interrupt};

#[entry]
fn foo() -> ! {
    loop {}
}

#[interrupt] //~ ERROR failed to resolve: use of undeclared type or module `interrupt`
fn USART1() {}
