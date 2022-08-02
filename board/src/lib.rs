//! A very simple, multi-board BSP for imxrt-rt-support examples.
#![no_std]

use imxrt_ral as ral;

cfg_if::cfg_if! {
    if #[cfg(feature = "teensy4")] {
        mod shared { pub mod imxrt10xx; }
        use shared::imxrt10xx::prepare_pit;

        mod teensy4;
        pub use teensy4::*;
    } else if #[cfg(feature = "imxrt1010evk")] {
        mod shared { pub mod imxrt10xx; }
        use shared::imxrt10xx::prepare_pit;

        mod imxrt1010evk;
        pub use imxrt1010evk::*;
    } else if #[cfg(feature = "imxrt1170evk-cm7")] {
        mod shared { pub mod imxrt11xx; }
        use shared::imxrt11xx::prepare_pit;

        mod imxrt1170evk_cm7;
        pub use imxrt1170evk_cm7::*;
    } else {
        compile_error!("No board feature selected!");
    }
}

pub struct Pit(&'static ral::pit::RegisterBlock);

impl Pit {
    fn new(pit: &ral::pit::RegisterBlock, timer_delay_microseconds: u32) -> Self {
        // Disable the PIT, just in case it was used by the boot ROM
        ral::write_reg!(ral::pit, pit, MCR, MDIS: 1);
        let timer = &pit.TIMER[0];
        // Reset channel 0 control; we'll use channel 0 for our timer
        ral::write_reg!(ral::pit::timer, timer, TCTRL, 0);
        // Set the counter value
        ral::write_reg!(ral::pit::timer, timer, LDVAL, timer_delay_microseconds);
        // Enable the PIT timer
        ral::modify_reg!(ral::pit, pit, MCR, MDIS: 0);
        Self(unsafe { core::mem::transmute(pit) })
    }
    pub fn blocking_delay(&mut self) {
        let timer = &self.0.TIMER[0];
        // Start counting!
        ral::write_reg!(ral::pit::timer, timer, TCTRL, TEN: 1);
        // Are we done?
        while ral::read_reg!(ral::pit::timer, timer, TFLG, TIF == 0) {}
        // We're done; clear the flag
        ral::write_reg!(ral::pit::timer, timer, TFLG, TIF: 1);
        // Turn off the timer
        ral::write_reg!(ral::pit::timer, timer, TCTRL, TEN: 0);
    }
    pub fn loop_with_interrupts(&mut self) {
        let timer = &self.0.TIMER[0];
        // Enable interrupts and start counting
        ral::write_reg!(ral::pit::timer, timer, TCTRL, TIE: 1);
        ral::modify_reg!(ral::pit::timer, timer, TCTRL, TEN: 1);
    }
    pub fn clear_interrupts(&mut self) {
        let timer = &self.0.TIMER[0];
        while ral::read_reg!(ral::pit::timer, timer, TFLG, TIF == 1) {
            ral::write_reg!(ral::pit::timer, timer, TFLG, TIF: 1);
        }
    }
}

unsafe impl Send for Pit {}

pub struct Led {
    offset: u32,
    port: &'static ral::gpio::RegisterBlock,
}

impl Led {
    fn new(offset: u32, port: &ral::gpio::RegisterBlock) -> Self {
        let led = Led {
            offset,
            port: unsafe { core::mem::transmute(port) },
        };
        ral::modify_reg!(ral::gpio, port, GDIR, |gdir| gdir | led.mask());
        led
    }
    fn mask(&self) -> u32 {
        1 << self.offset
    }
    pub fn set(&self) {
        ral::write_reg!(ral::gpio, self.port, DR_SET, self.mask());
    }

    pub fn clear(&self) {
        ral::write_reg!(ral::gpio, self.port, DR_CLEAR, self.mask());
    }

    pub fn toggle(&self) {
        ral::write_reg!(ral::gpio, self.port, DR_TOGGLE, self.mask());
    }
}

unsafe impl Send for Led {}

pub struct Resources {
    pub led: crate::Led,
    pub pit: crate::Pit,
}
