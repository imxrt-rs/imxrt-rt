#![no_main]
#![no_std]

extern crate imxrt_rt;
extern crate panic_halt;

use imxrt_rt::{entry, interrupt};

#[entry]
fn foo() -> ! {
    loop {}
}

#[allow(non_camel_case_types)]
enum interrupt {
    USART1,
}

#[interrupt(true)] //~ ERROR This attribute accepts no arguments
fn USART1() {}
