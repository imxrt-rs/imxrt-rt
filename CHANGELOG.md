# Changelog

## [Unreleased]

Add new MCU targets:

- imxrt1040

## [0.1.5] 2024-10-26

Add initial support for RT1180.

## [0.1.4] 2024-04-05

Add configurations to `RuntimeBuilder`:

- `stack_size_env_override`
- `heap_size_env_override`

Use these methods to define environment variables that can override the
stack / heap sizes.

## [0.1.3] 2023-10-01

Ensure that the runtime supports the GNU linker, `ld`.

## [0.1.2] 2023-09-08

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

[Unreleased]: https://github.com/imxrt-rs/imxrt-rt/compare/v0.1.5...HEAD
[0.1.5]: https://github.com/imxrt-rs/imxrt-rt/releases/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/imxrt-rs/imxrt-rt/releases/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/imxrt-rs/imxrt-rt/releases/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/imxrt-rs/imxrt-rt/releases/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/imxrt-rs/imxrt-rt/releases/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/imxrt-rs/imxrt-rt/releases/tag/v0.1.0
