//! Slowly blink an LED while blocking on a timer.
//!
//! Use this as the minimum-viable runtime support. You don't
//! need MCU-specific interrupts for this example.
//!
//! This example demonstrates how to register an exception
//! handler. See the API documentation for more information.

#![no_std]
#![no_main]

const PIT_PERIOD_US: u32 = 1_000_000;

#[imxrt_rt::entry]
fn main() -> ! {
    let board::Resources { mut pit, led, .. } = board::prepare(PIT_PERIOD_US).unwrap();
    loop {
        led.toggle();
        pit.blocking_delay();
    }
}

use imxrt_rt::exception;

#[exception]
unsafe fn DefaultHandler(_irqn: i16) {
    uh_oh()
}

#[exception]
unsafe fn HardFault(_: &imxrt_rt::ExceptionFrame) -> ! {
    uh_oh()
}

#[inline(never)]
fn uh_oh() -> ! {
    loop {
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst)
    }
}
