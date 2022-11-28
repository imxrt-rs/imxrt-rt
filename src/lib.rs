//! Runtime and startup support for i.MX RT processors.
//!
//! This crate builds on `cortex-m-rt` and adds support for i.MX RT processors.
//! Using this runtime crate, you can specify FlexRAM sizes and section allocations,
//! then use it to boot your i.MX RT processor.
//!
//! The crate achieves this with
//!
//! - a build-time API to define the memory map.
//! - a runtime library to configure the embedded processor.
//!
//! Both APIs are exposed from the same package. The interface changes depending on the
//! build environment.
//!
//! # Getting started
//!
//! Make sure you're familiar with [`cortex-m-rt`][cmrt] features. This crate re-exports
//! the `cortex-m-rt` interface. Use this interface to implement your program's entrypoint,
//! register exceptions, and interrupts. You should be familiar with specifing a linker
//! script for your embedded project.
//!
//! [cmrt]: https://docs.rs/cortex-m-rt/0.7.1/cortex_m_rt/
//!
//! # Dependencies
//!
//! In your embedded target, depend on `imxrt-rt` in both of
//!
//! - the `[dependencies]` section of your Cargo.toml
//! - the `[build-dependencies]` section of your Cargo.toml
//!
//! Use the same crate version in both locations. If you enable features, you must enable
//! features in both locations. See the features section for more information.
//!
//! ```text
//! [dependencies.imxrt-rt]
//! version = # $VERSION
//!
//! [build-dependencies.imxrt-rt]
//! version = # Same as $VERSION
//! ```
//!
//! # Linker script
//!
//! **Link against `imxrt-link.x`**, which is automatically made available on the linker search path.
//! Do not link against `link.x` from `cortex-m-rt`.
//!
//! You may change the name of the linker script by using the `RuntimeBuilder`.
//!
//! # Host configuration
//!
//! In your project, create a `build.rs` script that configures the runtime. The simplest `build.rs`
//! looks like this:
//!
//! ```no_run
//! use imxrt_rt::{Family, RuntimeBuilder};
//!
//! /// CHANGE ME depending on your board's flash size.
//! const FLASH_SIZE: usize = 16 * 1024 * 1024; // 16 MiB.
//! /// CHANGE ME depending on your board's chip.
//! const FAMILY: Family = Family::Imxrt1060;
//!
//! fn main() {
//!     RuntimeBuilder::from_flexspi(FAMILY, FLASH_SIZE)
//!         .build()
//!         .unwrap();
//! }
//! ```
//!
//! This script works for any i.MX RT 1060-based system that has 16 MiB of external flash.
//! Change the flash size and chip family based on your hardware. It uses the default configuration,
//! which tries to give a reasonable memory layout for all processors.
//! To understand the default configuration, see the [`RuntimeBuilder`] documentation.
//!
//! A more advanced runtime configuration looks like this:
//!
//! ```no_run
//! # use imxrt_rt::{Family, RuntimeBuilder};
//! use imxrt_rt::{FlexRamBanks, Memory};
//! # const FLASH_SIZE: usize = 16 * 1024 * 1024; // 16 MiB.
//! # const FAMILY: Family = Family::Imxrt1060;
//!
//! fn main() {
//!     RuntimeBuilder::from_flexspi(FAMILY, FLASH_SIZE)
//!         .flexram_banks(FlexRamBanks {
//!             ocram: 0,
//!             dtcm: FAMILY.flexram_bank_count() / 2 + 2,
//!             itcm: FAMILY.flexram_bank_count() / 2 - 2,
//!         })
//!         .text(Memory::Itcm)
//!         .vectors(Memory::Itcm)
//!         .rodata(Memory::Dtcm)
//!         .data(Memory::Dtcm)
//!         .bss(Memory::Dtcm)
//!         .uninit(Memory::Dtcm)
//!         .stack(Memory::Dtcm)
//!         .stack_size(4 * 1024)
//!         .heap(Memory::Dtcm)
//!         .heap_size(512)
//!         .build()
//!         .unwrap();
//! }
//! ```
//!
//! This configuration maximizes the TCM allocation by removing OCRAM blocks. It takes two
//! banks from ITCM, and gives them to DTCM. It ensures that all sections are allocated to
//! DTCM instead of OCRAM. It reduces the stack size, and reserves space for a small heap.
//!
//! No matter the configuration, the runtime ensures that all contents are copied from flash
//! into their respective locations before `main()` is called.
//!
//! # Target integration
//!
//! If your runtime uses flash, link against a FlexSPI configuration block (FCB) crate. The
//! crate is expected to export a `static FLEXSPI_CONFIGURATION_BLOCK` that describes how the
//! FlexSPI peripheral interacts with your external flash chip. If an FCB crate doesn't exist
//! for your hardware, you can use the [`imxrt-boot-gen` crate](https://docs.rs/imxrt-boot-gen/0.2.0/imxrt_boot_gen/)
//! to define one. See the [`teensy4-fcb` crate](https://docs.rs/teensy4-fcb/0.3.0/teensy4_fcb/)
//! for an example of an FCB crate that is compatible with this runtime.
//!
//! Finally, use `imxrt-rt` in your firmware just as you would use `cortex-m-rt`. See the [`cortex-m-rt`
//! documentation][cmrt] for examples.
//!
//! # Feature flags
//!
//! `imxrt-rt` supports the features available in `cortex-m-rt` version 0.7.2. If you enable a feature,
//! you must enable it in both the `[dependencies]` and `[build-dependencies]` section of your package
//! manifest. For example, if the `cortex-m-rt` `"device"` feature were needed, then enable this crate's
//! `"device"` feature in both places.
//!
//! ```text
//! [dependencies.imxrt-rt]
//! version = # $VERSION
//! features = ["device"]  # Add the feature here...
//!
//! [build-dependencies.imxrt-rt]
//! version = # Same as $VERSION
//! features = ["device"] # ... and here
//! ```
//!
//! # Limitations
//!
//! The crate considers the assignment of FlexRAM memory banks to ITCM/DTCM/OCRAM
//! an implementation detail. Additionally, the implementation does not care
//! about the assignment of memory bank power domains. This seems to matter most on
//! the 1050, which has the widest spread of bank-to-power domain assignment
//! (according to AN12077).
//!
//! There is no support for ECC on 1170. The runtime assumes that OCRAM and TCM ECC
//! is disabled, and that the corresponding memory banks can be used for OCRAM.
//!
//! The runtime installs a `cortex-m-rt` `pre_init` function to configure the runtime.
//! You cannot also define a `pre_init` function, and this crate does not support any
//! other mechanism for running code before `main()`.
//!
//! The implementation assumes all flash is FlexSPI.

#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std)]

cfg_if::cfg_if! {
    if #[cfg(all(target_arch = "arm", target_os = "none"))] {
        mod target;
        pub use target::*;
    } else {
        mod host;
        pub use host::*;
    }
}
