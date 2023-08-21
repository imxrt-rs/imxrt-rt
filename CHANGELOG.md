# Changelog

## [Unreleased]

Add section for device configuration data (DCD) in linker script. Users
can place their DCD in a section called `.dcd`. Consider using imxrt-dcd
as a convenient way to define a DCD.

## [0.1.1] 2023-02-14

Update to cortex-m-rt 0.7.3 to avoid certain miscompilation opportunities.
For more information, see the [cortex-m-rt advisory][cmrt-0.7.3].

[cmrt-0.7.3]: https://github.com/rust-embedded/cortex-m/discussions/469

Note that imxrt-rt 0.1.0 will no longer build. If you observe this error,
ensure that your build uses this imxrt-rt release.

## [0.1.0] 2022-12-02

First release. `imxrt-rt` provides a build-time API that defines a memory map,
as well as a runtime library that configures i.MX RT 10xx and 11xx processors.

[Unreleased]: https://github.com/imxrt-rs/imxrt-rt/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/imxrt-rs/imxrt-rt/releases/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/imxrt-rs/imxrt-rt/releases/tag/v0.1.0
