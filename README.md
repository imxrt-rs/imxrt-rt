# imxrt-rt

Runtime and startup support for i.MX RT processors.

This crate builds on `cortex-m-rt` and adds support for i.MX RT
processors. Using this runtime crate, you can specify FlexRAM sizes and
section allocations, then use it to boot your i.MX RT processor.

The crate achieves this with

-   a build-time API to define the memory map.
-   a runtime library to configure the embedded processor.

To learn how to use this crate in your firmware, see the crate
documentation. To try the runtime on hardware, see [the `board`
documentation].

  [the `board` documentation]: board/README.md

## Development

Run automated tests like this:

    cargo test --tests
    cargo test --doc
    cargo test --tests -- --ignored

If you have `pyOCD` available, you can check the effects of the runtime
initialization routine with GDB:

    pyocd gdb --target=$YOUR_TARGET
    arm-none-eabi-gdb < cmds.gdb

Make sure that the register values make sense for your target.

## License

Licensed under either of

-   [Apache License, Version 2.0] ([LICENSE-APACHE])
-   [MIT License] ([LICENSE-MIT])

at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms
or conditions.

  [Apache License, Version 2.0]: http://www.apache.org/licenses/LICENSE-2.0
  [LICENSE-APACHE]: ./LICENSE-APACHE
  [MIT License]: http://opensource.org/licenses/MIT
  [LICENSE-MIT]: ./LICENSE-MIT
