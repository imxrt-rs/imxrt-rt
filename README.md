# imxrt-rt

An API-compatible fork of the `cortex-m-rt` crate that describes the system's
memory layout, startup sequence, and interrupt table. The runtime crate let's a
user write a normal `main()` function. See the "Why" notes to learn why this is
a fork of the `cortex-m-rt` crate.

## Why

An embedded Rust developer might use the
[`cortex-m-rt`](https://crates.io/crates/cortex-m-rt) to bootstrap a Cortex-M
system. However, [#164](https://github.com/rust-embedded/cortex-m-rt/issues/164)
notes that the `cortex-m-rt` crate cannot yet support devices with custom memory
layouts. The iMXRT106x is one of the systems with a custom memory layout; in
particular, we have tightly-coupled memory (TCM) regions for instructions (ITCM)
and data (DTCM). We also need to place special arrays (the FCB) in memory in
order to properly boot. Given these requirements, we need a custom runtime crate
that can initialize the system.

The `imxrt-rt` crate is a fork of the `cortex-m-rt` crate that is customized to
support a minimal iMXRT1062 startup and runtime. Like the `cortex-m-rt` crate,
the `imxrt-rt` crate

- populates the vector table for correct booting and exception / interrupt
- dispatch initializes static variables enables the FPU (since we're a
- `thumbv7em-none-eabihf` device)

The `imxrt-rt` crate goes a step further in its startup functionality:

- provides the required firmware configuration block (FCB) placement and image
- vector table (IVT) in order to start the iMXRT106x initialize the TCM memory
- regions configures instruction and data caches based on the TCM regions

Just as the `cortex-m-rt` crate will call a user's `main()` function, the
`imxrt-rt` completes by calling a user's `main()`. The `imxrt-rt` crate also
exposes the `#[interrupt]`, `#[exception]`, and `#[entry]` macros for decorating
interrupt handlers, exception handlers, and the program entrypoint,
respectively. Note that, as of this writing, `#[pre_init]` is not supported.

To support compatibility with the `cortex-m-rt` crate, the `imxrt-rt` crate uses
the same link sections as the `cortex-m-rt` crate. However, the `imxrt-rt` crate
may locate memory in different regions. Specifically, all instructions are
placed into ITCM, and all data is placed into DTCM.

It is our hope that the `imxrt-rt` crate can be transparently replaced with the
`cortext-m-rt` crate once the necessary features are available. If you think
that the `imxrt-rt` crate is be diverging from the `cortex-m-rt` crate and might
miss that goal, please file an issue!

## Acknowledgements and References

- The [Teensy 4](https://www.pjrc.com/store/teensy40.html) is wonderful, and
  that's thanks to the hard work of PJRC and friends. We can find the Teensy
  code used in the Arduino plugins
- [here](https://github.com/PaulStoffregen/cores). The code greatly influenced
  this library.  I'm not the only developer tackling the "Rust on Teensy 4"
  challenge. Check out mpasternacki's work
- [here](https://gitlab.com/teensy-rs/teensy-4) as an alternative approach
  towards the same problem.  The Rust Cortex M team, specifically the
  [`cortex-m-rt`](https://github.com/rust-embedded/cortex-m-rt) crate.
- [`teensy4-rs`](https://github.com/mciantyre/teensy4-rs) Project which this is
  entirely based off.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
