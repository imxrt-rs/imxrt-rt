//! Slowly blink an LED when a timer interrupt fires.
//!
//! This example is a little more complex, and shows that the
//! vector table is placed and known to the processor.

#![no_std]
#![no_main]

use imxrt_rt as _;

/// A static that forces this binary to include a .data section.
/// This is checked in an automated test.
static mut DATA: u32 = 5;

#[rtic::app(device = board::rtic_support, peripherals = false)]
mod app {
    const PIT_PERIOD_US: u32 = 1_000_000;

    #[local]
    struct Local {
        led: board::Led,
        pit: board::Pit,
    }

    #[shared]
    struct Shared {}

    #[init]
    fn init(_: init::Context) -> (Shared, Local, init::Monotonics) {
        let board::Resources { mut pit, led, .. } = board::prepare(PIT_PERIOD_US).unwrap();
        pit.loop_with_interrupts();
        led.set();
        (Shared {}, Local { led, pit }, init::Monotonics())
    }

    #[task(binds = PIT, local = [led, pit])]
    fn pit(cx: pit::Context) {
        unsafe { crate::DATA += 1 };
        cx.local.led.toggle();
        cx.local.pit.clear_interrupts();
    }
}
