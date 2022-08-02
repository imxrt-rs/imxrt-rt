`board` provides a thin board support package for `imxrt-rt`. The
package provides cross-board compatibility for all `imxrt-rt` hardware
examples. It supports `imxrt-rt` development and testing, and is not
intended as a general BSP.

`board` supports

-   Teensy 4.0 and Teensy 4.1 boards with the `teensy4` feature.
-   the IMXRT1010EVK board with the `imxrt1010evk` feature.
-   the Cortex M7 on the IMXRT1170EVK with the `imxrt1170evk-cm7`
    feature.

When using any NXP EVK, make sure that your boot device is FlexSPI.
Consult your board's hardware user guide for more information.

## Board configurations

The examples in this repository are very basic. They demonstrate a
working runtime by blinking an LED. They also configure timer interrupts
to show that the vector table is placed and registered correctly. They
only use `imxrt-ral` to access registers.

Boards simply specify an LED. See the relevant module in `board/src/`
for more information.

`build.rs` configures the runtime for each board. You can change this to
explore different runtime configurations.

## Building hardware examples

Hardware examples for `imxrt-rt` depend on `board` and a board
selection. This section describes how to build an example for your
board. It focuses on building examples for the Teensy 4, but the concept
generalizes for all supported boards.

To build the `blink-blocking` example for a Teensy 4, run the command
from the repo's root:

    cargo build --example=blink-blocking --features=board/teensy4 --target=thumbv7em-none-eabihf

Generally, you select the example with `--example`, and specify the
board with `--features=board/[your-board]`. To build the same example
for the IMXRT1010EVK, change `--features=board/teensy4` to
`--features=board/imxrt1010evk`.

To build an RTIC-based example, enable the `rtic` feature of `board`.

Artifacts are available under
`target/thumbv7em-none-eabihf/[debug|release]/examples`. Keep this in
mind when flashing your board.

## Flashing hardware examples

The tools required to flash an example depend on the board you're using.
This section recommends tooling to flash hardware examples on your
board.

### NXP IMXRT EVKs

If you're using an NXP IMXRT EVK, you can use any of the following to
flash your board.

-   [`pyOCD`] supports all i.MX RT 10xx and 11xx boards.
-   [`probe-rs` tools] only support i.MX RT 10xx boards. These tools
    include
    -   [`probe-run`]
    -   [`cargo-flash`]
    -   [`cargo-embed`]

See each tool's documentation to understand its usage. To make some
tooling integration easier, see the Tips and Tricks section near the end
of this document.

  [`pyOCD`]: https://pyocd.io
  [`probe-rs` tools]: https://probe.rs
  [`probe-run`]: https://github.com/knurling-rs/probe-run
  [`cargo-flash`]: https://github.com/probe-rs/cargo-flash
  [`cargo-embed`]: https://github.com/probe-rs/cargo-embed

### Teensy 4

If you're using a Teensy 4 board, you'll need all of the following:

-   An `objcopy` capable of transforming ELF files into Intel HEX.
    Consider using `rust-objcopy` provided by [`cargo-binutils`]. The
    rest of this documentation assumes you're using `cargo-binutils`.
-   Either a build of [`teensy_loader_cli`], or the [Teensy Loader
    Application]. The latter is available with the Teensyduino add-ons.

After building your example, use `rust-objcopy` to convert the program
into HEX. For the `blink-blocking` example above, that command resembles

    rust-objcopy -O ihex target/thumbv7em-none-eabihf/debug/examples/blink-blocking blink-blocking.hex

Finally, load the HEX file onto your board using your preferred loader.

  [`cargo-binutils`]: https://github.com/rust-embedded/cargo-binutils
  [`teensy_loader_cli`]: https://github.com/PaulStoffregen/teensy_loader_cli
  [Teensy Loader Application]: https://www.pjrc.com/teensy/loader.html

## Tips and tricks

If you're using `probe-run` or `pyOCD` to flash an EVK, use the tool as
a runner. See the [Cargo Configuration] documentation for more
information. Please do not check your runner setting into the
repository; consider using environment variables or hierarchical
configuration files to configure your runner and any other useful
command aliases.

  [Cargo Configuration]: https://doc.rust-lang.org/cargo/reference/config.html

## Adding a new board

Define a new module in `board/src/` that describes your board's LED. Use
the existing examples as your guide.

Add a new feature to `board/Cargo.toml` for your board. Link any
additional dependencies for your board, like FCB crates and panic
handlers. If an FCB crate does not exist for your board, you can define
the FCB within your newly-added module.

Update `board/build.rs` to configure a runtime for your chip.
