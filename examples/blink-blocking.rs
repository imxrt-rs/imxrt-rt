//! Slowly blink an LED while blocking on a timer.
//!
//! Use this as the minimum-viable runtime support. You don't
//! need interrupts for this example.

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
