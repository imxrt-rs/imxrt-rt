//! Host-side configurations for the target.
//!
//! See [`RuntimeBuilder::build`] to understand the linker script generation
//! steps.

// Please explicitly match all `Family` variants. If someone wants to add
// a new `Family`, this will show help them find all the settings they need
// to consider.
#![warn(clippy::wildcard_enum_match_arm)]

use std::{
    env,
    fmt::Display,
    fs,
    io::{self, Write},
    path::PathBuf,
};

/// Memory partitions.
///
/// Use with [`RuntimeBuilder`] to specify the placement of sections
/// in the final program. Note that the `RuntimeBuilder` only does limited
/// checks on memory placements. Generally, it's OK to place data in ITCM,
/// and instructions in DTCM; however, this isn't recommended for optimal
/// performance.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Memory {
    /// Place the section in (external) flash.
    ///
    /// Reads and writes are translated into commands on an external
    /// bus, like FlexSPI.
    Flash,
    /// Place the section in data tightly coupled memory (DTCM).
    Dtcm,
    /// Place the section in instruction tightly coupled memory (ITCM).
    Itcm,
    /// Place the section in on-chip RAM (OCRAM).
    ///
    /// If your chip includes dedicated OCRAM memory, the implementation
    /// utilizes that OCRAM before utilizing any FlexRAM OCRAM banks.
    Ocram,
}

/// The FlexSPI peripheral that interfaces your flash chip.
///
/// The [`RuntimeBuilder`] selects `FlexSpi1` for nearly all chip
/// families. However, it selects `FlexSpi2` for the 1064 in order
/// to utilize its on-board flash. You can override the selection
/// using [`RuntimeBuilder::flexspi()`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexSpi {
    /// Interface flash using FlexSPI 1.
    FlexSpi1,
    /// Interface flash using FlexSPI 2.
    FlexSpi2,
}

impl FlexSpi {
    fn family_default(family: Family) -> Self {
        match family {
            Family::Imxrt1064 => FlexSpi::FlexSpi2,
            Family::Imxrt1010
            | Family::Imxrt1015
            | Family::Imxrt1020
            | Family::Imxrt1040
            | Family::Imxrt1050
            | Family::Imxrt1060
            | Family::Imxrt1160
            | Family::Imxrt1170
            | Family::Imxrt1180 => FlexSpi::FlexSpi1,
        }
    }
    fn start_address(self, family: Family) -> Option<u32> {
        match (self, family) {
            // FlexSPI1, 10xx
            (
                FlexSpi::FlexSpi1,
                Family::Imxrt1010
                | Family::Imxrt1015
                | Family::Imxrt1020
                | Family::Imxrt1040
                | Family::Imxrt1050
                | Family::Imxrt1060
                | Family::Imxrt1064,
            ) => Some(0x6000_0000),
            // FlexSPI2 not available on 10xx families
            (
                FlexSpi::FlexSpi2,
                Family::Imxrt1010 | Family::Imxrt1015 | Family::Imxrt1020 | Family::Imxrt1050,
            ) => None,
            // FlexSPI 2 available on 10xx families
            (FlexSpi::FlexSpi2, Family::Imxrt1040 | Family::Imxrt1060 | Family::Imxrt1064) => {
                Some(0x7000_0000)
            }
            // 11xx support
            (FlexSpi::FlexSpi1, Family::Imxrt1160) => Some(0x3000_0000),
            (FlexSpi::FlexSpi2, Family::Imxrt1160) => Some(0x6000_0000),
            (FlexSpi::FlexSpi1, Family::Imxrt1170) => Some(0x3000_0000),
            (FlexSpi::FlexSpi2, Family::Imxrt1170) => Some(0x6000_0000),
            (FlexSpi::FlexSpi1, Family::Imxrt1180) => Some(0x2800_0000),
            (FlexSpi::FlexSpi2, Family::Imxrt1180) => Some(0x0400_0000),
        }
    }
    fn supported_for_family(self, family: Family) -> bool {
        self.start_address(family).is_some()
    }
}

impl Display for Memory {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Flash => f.write_str("FLASH"),
            Self::Itcm => f.write_str("ITCM"),
            Self::Dtcm => f.write_str("DTCM"),
            Self::Ocram => f.write_str("OCRAM"),
        }
    }
}

/// Define an alias for `name` that maps to a memory block named `placement`.
fn region_alias(output: &mut dyn Write, name: &str, placement: Memory) -> io::Result<()> {
    writeln!(output, "REGION_ALIAS(\"REGION_{name}\", {placement});")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FlashOpts {
    size: usize,
    offset: u32,
    flexspi: FlexSpi,
    boot_header: bool,
}

impl FlashOpts {
    /// Produce the flash address of the image within
    /// FlexSPI memory.
    fn flash_origin(&self, family: Family) -> Option<u32> {
        self.flexspi
            .start_address(family)
            .map(|start_address| start_address + self.offset)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EnvOverride {
    default: usize,
    env: Option<String>,
}

impl EnvOverride {
    const fn new(default: usize) -> Self {
        Self { default, env: None }
    }
    fn set_env_key(&mut self, key: String) {
        self.env = Some(key);
    }
    fn read(&self) -> Result<usize, Box<dyn std::error::Error>> {
        if let Some(env) = &self.env {
            // If the user sets multiple environment variables for the same runtime
            // property (like stack, heap), we will only re-run when the variable
            // we care about changes. An example might help:
            //
            //      let mut bldr = RuntimeBuilder::from_flexspi(/* ... */);
            //      bldr.stack_size_env_override("COMMON_STACK");
            //      if special_condition() {
            //          bldr.stack_size_env_override("SPECIAL_STACK");
            //      }
            //
            // If we take the branch, then we re-run the build if SPECIAL_STACK
            // changes. Otherwise, we re-run if COMMON_STACK changes.
            //
            // I previously put this into `set_env_key`. That would mean we re-run
            // the build if _either_ enviroment variable changes. But then I thought
            // about the user who writes their build script like
            //
            //      if special_condition() {
            //          bldr.stack_size_env_override("SPECIAL_STACK");
            //      } else {
            //          bldr.stack_size_env_override("COMMON_STACK");
            //      }
            //
            // and how that would introduce different re-run behavior than the first
            // example, even though the variable selection is the same. Coupling the
            // re-run behavior to the variable selection behavior seems less surprising.
            println!("cargo:rerun-if-env-changed={env}");
        }

        if let Some(val) = self.env.as_ref().and_then(|key| env::var(key).ok()) {
            let val = if val.ends_with('k') || val.ends_with('K') {
                val[..val.len() - 1].parse::<usize>()? * 1024
            } else {
                val.parse::<usize>()?
            };
            Ok(val)
        } else {
            Ok(self.default)
        }
    }
}

/// Builder for the i.MX RT runtime.
///
/// `RuntimeBuilder` let you assign sections to memory regions. It also lets
/// you partition FlexRAM DTCM/ITCM/OCRAM. Call [`build()`](RuntimeBuilder::build) to commit the
/// runtime configuration.
///
/// # Behaviors
///
/// The implementation tries to place the stack in the lowest-possible memory addresses.
/// This means the stack will grow down into reserved memory below DTCM and OCRAM for most
/// chip families. The outlier is the 1170, where the stack will grow into OCRAM backdoor for
/// the CM4 coprocessor. Be careful here...
///
/// Similarly, the implementation tries to place the heap in the highest-possible memory
/// addresses. This means the heap will grow up into reserved memory above DTCM and OCRAM
/// for most chip families.
///
/// The vector table requires a 1024-byte alignment. The vector table's placement is prioritized
/// above all other sections, except the stack. If placing the stack and vector table in the
/// same section (which is the default behavior), consider keeping the stack size as a multiple
/// of 1 KiB to minimize internal fragmentation.
///
/// # Default values
///
/// The example below demonstrates the default `RuntimeBuilder` memory placements,
/// stack sizes, heap sizes, and additional configurations.
///
/// ```
/// use imxrt_rt::{Family, RuntimeBuilder, Memory};
///
/// const FLASH_SIZE: usize = 16 * 1024;
/// let family = Family::Imxrt1060;
///
/// let mut b = RuntimeBuilder::from_flexspi(family, FLASH_SIZE);
/// // FlexRAM layout represent default fuse values.
/// b.flexram_layout(&family.default_flexram_layout());
/// b.text(Memory::Itcm);    // Copied from flash.
/// b.rodata(Memory::Ocram); // Copied from flash.
/// b.data(Memory::Ocram);   // Copied from flash.
/// b.vectors(Memory::Dtcm); // Copied from flash.
/// b.bss(Memory::Ocram);
/// b.uninit(Memory::Ocram);
/// b.stack(Memory::Dtcm);
/// b.stack_size(8 * 1024);  // 8 KiB stack.
/// b.heap(Memory::Dtcm);    // Heap in DTCM...
/// b.heap_size(0);          // ...but no space given to the heap.
/// b.linker_script_name("imxrt-link.x");
/// b.device_script_name("device.x");
///
/// assert_eq!(b, RuntimeBuilder::from_flexspi(family, FLASH_SIZE));
/// ```
///
/// Note that, if you specify [`FlexRamBanks`], the corresponding
/// layout may be different than the default layout.
///
/// ```
/// # use imxrt_rt::{Family, RuntimeBuilder, Memory};
/// # const FLASH_SIZE: usize = 16 * 1024;
/// # let family = Family::Imxrt1060;
/// let mut b = RuntimeBuilder::from_flexspi(family, FLASH_SIZE);
/// b.flexram_banks(family.default_flexram_banks());
/// assert_ne!(b, RuntimeBuilder::from_flexspi(family, FLASH_SIZE));
/// ```
///
/// # Environment overrides
///
/// Certain memory regions, like the stack and heap, can be sized using environment
/// variables. As the provider of the runtime, you can use `*_env_override` methods
/// to select the environment variable(s) that others may use to set the size, in bytes,
/// for these memory regions.
///
/// The rest of this section describes how environment variables interact with other
/// methods on this builder. Although the examples use stack size, the concepts apply
/// to all regions that can be sized with environment variables.
///
/// ```no_run
/// # use imxrt_rt::{Family, RuntimeBuilder, Memory};
/// # const FLASH_SIZE: usize = 16 * 1024;
/// # let family = Family::Imxrt1060;
/// RuntimeBuilder::from_flexspi(family, FLASH_SIZE)
///     .stack_size_env_override("YOUR_STACK_SIZE")
///     // ...
///     # .build().unwrap();
/// ```
///
/// In the above example, if a user set an environment variable `YOUR_STACK_SIZE=1024`, then
/// the runtime's stack size is 1024. Otherwise, the stack size is the default stack size.
///
/// > As a convenience, a user can use a `k` or `K` suffix to specify multiples of 1024 bytes.
/// > For example, the environment variables `YOUR_STACK_SIZE=4k` and `YOUR_STACK_SIZE=4K` are
/// > each equivalent to `YOUR_STACK_SIZE=4096`.
///
/// ```no_run
/// # use imxrt_rt::{Family, RuntimeBuilder, Memory};
/// # const FLASH_SIZE: usize = 16 * 1024;
/// # let family = Family::Imxrt1060;
/// RuntimeBuilder::from_flexspi(family, FLASH_SIZE)
///     .stack_size_env_override("YOUR_STACK_SIZE")
///     .stack_size(2048)
///     // ...
///     # .build().unwrap();
///
/// RuntimeBuilder::from_flexspi(family, FLASH_SIZE)
///     .stack_size(2048)
///     .stack_size_env_override("YOUR_STACK_SIZE")
///     // ...
///     # .build().unwrap();
/// ```
///
/// In the above example, the two builders produce the same runtime. The builder
/// selects the stack size from the environment variable, if available. Otherwise,
/// the stack size is 2048 bytes. The call order is irrelevant, since the builder
/// doesn't consult the environment until you invoke [`build()`](Self::build).
///
/// ```no_run
/// # use imxrt_rt::{Family, RuntimeBuilder, Memory};
/// # const FLASH_SIZE: usize = 16 * 1024;
/// # let family = Family::Imxrt1060;
/// RuntimeBuilder::from_flexspi(family, FLASH_SIZE)
///     .stack_size_env_override("INVALIDATED")
///     .stack_size_env_override("YOUR_STACK_SIZE")
///     // ...
///     # .build().unwrap();
/// ````
///
/// In the above example, `YOUR_STACK_SIZE` invalidates the call with `INVALIDATED`.
/// Therefore, `YOUR_STACK_SIZE` controls the stack size, if set. Otherwise, the stack
/// size is the default stack size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeBuilder {
    family: Family,
    flexram_layout: Vec<FlexRamKind>,
    text: Memory,
    rodata: Memory,
    data: Memory,
    vectors: Memory,
    bss: Memory,
    uninit: Memory,
    stack: Memory,
    stack_size: EnvOverride,
    heap: Memory,
    heap_size: EnvOverride,
    flash_opts: Option<FlashOpts>,
    linker_script_name: String,
    device_script_name: String,
}

const DEFAULT_LINKER_SCRIPT_NAME: &str = "imxrt-link.x";
const DEFAULT_DEVICE_SCRIPT_NAME: &str = "device.x";

impl RuntimeBuilder {
    /// Creates a runtime that can execute and load contents from
    /// FlexSPI flash.
    ///
    /// `flash_size` is the size of your flash component, in bytes.
    pub fn from_flexspi(family: Family, flash_size: usize) -> Self {
        Self {
            family,
            flexram_layout: family.default_flexram_layout(),
            text: Memory::Itcm,
            rodata: Memory::Ocram,
            data: Memory::Ocram,
            vectors: Memory::Dtcm,
            bss: Memory::Ocram,
            uninit: Memory::Ocram,
            stack: Memory::Dtcm,
            stack_size: EnvOverride::new(8 * 1024),
            heap: Memory::Dtcm,
            heap_size: EnvOverride::new(0),
            flash_opts: Some(FlashOpts {
                size: flash_size,
                offset: 0,
                boot_header: true,
                flexspi: FlexSpi::family_default(family),
            }),
            linker_script_name: DEFAULT_LINKER_SCRIPT_NAME.into(),
            device_script_name: DEFAULT_DEVICE_SCRIPT_NAME.into(),
        }
    }

    /// Allocate a flash partition for this program to be booted by your software.
    ///
    /// `partition_size` is the size of the flash allocation, in bytes, for this
    /// program. `partition_offset` describes the byte offset where the partition
    /// starts. The offset is from the start of the FlexSPI memory region.
    ///
    /// The program constructed at this flash location cannot be booted by NXP's boot
    /// ROM. You should bring your own software to execute this program. Note that
    /// [the runtime behaviors](RuntimeBuilder) ensure that the vector table is placed
    /// in flash at the given `partition_offset`.
    ///
    /// To compute a partition offset from two absolute flash addresses, use
    /// [`Family::flexspi_start_addr`] to learn the FlexSPI starting address.
    pub fn in_flash(family: Family, partition_size: usize, partition_offset: u32) -> Self {
        Self {
            family,
            flexram_layout: family.default_flexram_layout(),
            text: Memory::Itcm,
            rodata: Memory::Ocram,
            data: Memory::Ocram,
            vectors: Memory::Dtcm,
            bss: Memory::Ocram,
            uninit: Memory::Ocram,
            stack: Memory::Dtcm,
            stack_size: EnvOverride::new(8 * 1024),
            heap: Memory::Dtcm,
            heap_size: EnvOverride::new(0),
            flash_opts: Some(FlashOpts {
                size: partition_size,
                offset: partition_offset,
                boot_header: false,
                flexspi: FlexSpi::family_default(family),
            }),
            linker_script_name: DEFAULT_LINKER_SCRIPT_NAME.into(),
            device_script_name: DEFAULT_DEVICE_SCRIPT_NAME.into(),
        }
    }

    /// Create a runtime that executes from RAM.
    pub fn from_ram(family: Family) -> Self {
        Self {
            family,
            flexram_layout: family.default_flexram_layout(),
            text: Memory::Itcm,
            rodata: Memory::Ocram,
            data: Memory::Ocram,
            vectors: Memory::Dtcm,
            bss: Memory::Ocram,
            uninit: Memory::Ocram,
            stack: Memory::Dtcm,
            stack_size: EnvOverride::new(8 * 1024),
            heap: Memory::Dtcm,
            heap_size: EnvOverride::new(0),
            flash_opts: None,
            linker_script_name: DEFAULT_LINKER_SCRIPT_NAME.into(),
            device_script_name: DEFAULT_DEVICE_SCRIPT_NAME.into(),
        }
    }

    /// Set the FlexRAM bank allocation.
    ///
    /// Use this to customize the sizes of DTCM, ITCM, and OCRAM.
    /// See the `FlexRamBanks` documentation for requirements on the
    /// bank allocations.
    pub fn flexram_banks(&mut self, flexram_banks: FlexRamBanks) -> &mut Self {
        self.flexram_layout(&flexram_banks.to_flexram_layout())
    }

    /// Set the FlexRAM bank layout.
    ///
    /// Use this to customize the sizes of DTCM, ITCM, and OCRAM.
    /// This also gives control of the bank assignment in the FlexRAM
    /// controller.
    pub fn flexram_layout(&mut self, flexram_layout: &[FlexRamKind]) -> &mut Self {
        self.flexram_layout = Vec::from(flexram_layout);
        self
    }

    /// Set the memory placement for code.
    pub fn text(&mut self, memory: Memory) -> &mut Self {
        self.text = memory;
        self
    }
    /// Set the memory placement for read-only data.
    pub fn rodata(&mut self, memory: Memory) -> &mut Self {
        self.rodata = memory;
        self
    }
    /// Set the memory placement for mutable data.
    pub fn data(&mut self, memory: Memory) -> &mut Self {
        self.data = memory;
        self
    }
    /// Set the memory placement for the vector table.
    pub fn vectors(&mut self, memory: Memory) -> &mut Self {
        self.vectors = memory;
        self
    }
    /// Set the memory placement for zero-initialized data.
    pub fn bss(&mut self, memory: Memory) -> &mut Self {
        self.bss = memory;
        self
    }
    /// Set the memory placement for uninitialized data.
    pub fn uninit(&mut self, memory: Memory) -> &mut Self {
        self.uninit = memory;
        self
    }
    /// Set the memory placement for stack memory.
    pub fn stack(&mut self, memory: Memory) -> &mut Self {
        self.stack = memory;
        self
    }
    /// Set the size, in bytes, of the stack.
    pub fn stack_size(&mut self, bytes: usize) -> &mut Self {
        self.stack_size.default = bytes;
        self
    }
    /// Let end users override the stack size using an environment variable.
    ///
    /// See the [environment overrides](Self#environment-overrides) documentation
    /// for more information.
    pub fn stack_size_env_override(&mut self, key: impl AsRef<str>) -> &mut Self {
        self.stack_size.set_env_key(key.as_ref().into());
        self
    }
    /// Set the memory placement for the heap.
    ///
    /// Note that the default heap has no size. Use [`heap_size`](Self::heap_size)
    /// to allocate space for a heap.
    pub fn heap(&mut self, memory: Memory) -> &mut Self {
        self.heap = memory;
        self
    }
    /// Set the size, in bytes, of the heap.
    pub fn heap_size(&mut self, bytes: usize) -> &mut Self {
        self.heap_size.default = bytes;
        self
    }
    /// Let end users override the heap size using an environment variable.
    ///
    /// See the [environment overrides](Self#environment-overrides) documentation
    /// for more information.
    pub fn heap_size_env_override(&mut self, key: impl AsRef<str>) -> &mut Self {
        self.heap_size.set_env_key(key.as_ref().into());
        self
    }
    /// Set the FlexSPI peripheral that interfaces flash.
    ///
    /// See the [`FlexSpi`] to understand the default values.
    /// If this builder is not configuring a flash-loaded runtime, this
    /// call is silently ignored.
    pub fn flexspi(&mut self, peripheral: FlexSpi) -> &mut Self {
        if let Some(flash_opts) = &mut self.flash_opts {
            flash_opts.flexspi = peripheral;
        }
        self
    }

    /// Override the boot header configuration.
    ///
    /// By default, a runtime constructed using [`from_flexspi`](Self::from_flexspi) includes a boot header
    /// for compatibility with NXP's boot ROM. However, a runtime constructed using [`in_flash`](Self::in_flash)
    /// lacks this boot header; you're expected to bring your own bootloader.
    ///
    /// This call lets you change that default behavior. If this builder is not configuring a flash-loaded
    /// runtime, this is silently ignored.
    pub fn boot_header(&mut self, boot_header: bool) -> &mut Self {
        if let Some(flash_opts) = &mut self.flash_opts {
            flash_opts.boot_header = boot_header;
        }
        self
    }

    /// Set the name of the linker script file.
    ///
    /// You can use this to customize the linker script name for your users.
    /// See the [crate-level documentation](crate#linker-script) for more
    /// information.
    pub fn linker_script_name(&mut self, name: &str) -> &mut Self {
        self.linker_script_name = name.into();
        self
    }

    /// Set the name of the device's linker file that's included by the
    /// runtime's linker script.
    ///
    /// By default, the device PAC is expected to place a file named
    /// `device.x` on the linker search path. The runtime's linker script
    /// includes that `device.x` file when the "device" crate feature is
    /// enabled.
    ///
    /// This method lets you change the name of that included file.
    pub fn device_script_name(&mut self, name: &str) -> &mut Self {
        self.device_script_name = name.into();
        self
    }

    /// Commit the runtime configuration.
    ///
    /// `build()` ensures that the generated linker script is available to the
    /// linker.
    ///
    /// # Errors
    ///
    /// The implementation ensures that your chip can support the FlexRAM bank
    /// allocation. An invalid allocation is signaled by an error.
    ///
    /// Returns an error if any of the following sections are placed in flash:
    ///
    /// - data
    /// - vectors
    /// - bss
    /// - uninit
    /// - stack
    /// - heap
    ///
    /// The implementation may rely on the _linker_ to signal other errors.
    /// For example, suppose a runtime configuration with no ITCM banks. If a
    /// section is placed in ITCM, that error could be signaled here, or through
    /// the linker. No matter the error path, the implementation ensures that there
    /// will be an error.
    pub fn build(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Since `build` is called from a build script, the output directory
        // represents the path to the _user's_ crate.
        let out_dir = PathBuf::from(env::var("OUT_DIR")?);
        println!("cargo:rustc-link-search={}", out_dir.display());

        // The main linker script expects to INCLUDE this file. This file
        // uses region aliases to associate region names to actual memory
        // regions (see the Memory enum).
        let mut in_memory = Vec::new();
        self.write_linker_script(&mut in_memory)?;
        fs::write(out_dir.join(&self.linker_script_name), &in_memory)?;
        Ok(())
    }

    /// Write the generated linker script into the provided writer.
    ///
    /// Use this if you want more control over where the generated linker script
    /// ends up. Otherwise, you should prefer [`build()`](Self::build) for an
    /// easier experience.
    ///
    /// Unlike `build()`, this method does not ensure that the linker script is
    /// available to the linker. Additionally, this method does not consider
    /// the value set by [`linker_script_name`](Self::linker_script_name).
    ///
    /// # Errors
    ///
    /// See [`build()`](Self::build) to understand the possible errors.
    fn write_linker_script(
        &self,
        writer: &mut dyn Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.check_configurations()?;

        if let Some(flash_opts) = &self.flash_opts {
            write_flash_memory_map(writer, self.family, flash_opts, &self.flexram_layout)?;

            if flash_opts.boot_header {
                let boot_header_x = match self.family {
                    Family::Imxrt1010
                    | Family::Imxrt1015
                    | Family::Imxrt1020
                    | Family::Imxrt1040
                    | Family::Imxrt1050
                    | Family::Imxrt1060
                    | Family::Imxrt1064
                    | Family::Imxrt1160
                    | Family::Imxrt1170 => include_bytes!("host/imxrt-boot-header.x").as_slice(),
                    Family::Imxrt1180 => include_bytes!("host/imxrt-boot-header-1180.x").as_slice(),
                };
                writer.write_all(boot_header_x)?;
            }
        } else {
            write_ram_memory_map(writer, self.family, &self.flexram_layout)?;
        }

        if cfg!(feature = "device") {
            writeln!(writer, "INCLUDE {}", self.device_script_name)?;
        }

        // Keep these alias names in sync with the primary linker script.
        // The main linker script uses these region aliases for placing
        // sections. Then, the user specifies the actual placement through
        // the builder. This saves us the step of actually generating SECTION
        // commands.
        region_alias(writer, "TEXT", self.text)?;
        region_alias(writer, "VTABLE", self.vectors)?;
        region_alias(writer, "RODATA", self.rodata)?;
        region_alias(writer, "DATA", self.data)?;
        region_alias(writer, "BSS", self.bss)?;
        region_alias(writer, "UNINIT", self.uninit)?;

        region_alias(writer, "STACK", self.stack)?;
        region_alias(writer, "HEAP", self.heap)?;
        // Used in the linker script and / or target code.
        writeln!(writer, "__stack_size = {:#010X};", self.stack_size.read()?)?;
        writeln!(writer, "__heap_size = {:#010X};", self.heap_size.read()?)?;

        if self.flash_opts.is_some() {
            // Runtime will see different VMA and LMA, and copy the sections.
            region_alias(writer, "LOAD_VTABLE", Memory::Flash)?;
            region_alias(writer, "LOAD_TEXT", Memory::Flash)?;
            region_alias(writer, "LOAD_RODATA", Memory::Flash)?;
            region_alias(writer, "LOAD_DATA", Memory::Flash)?;
        } else {
            // When the VMA and LMA are equal, the runtime performs no copies.
            region_alias(writer, "LOAD_VTABLE", self.vectors)?;
            region_alias(writer, "LOAD_TEXT", self.text)?;
            region_alias(writer, "LOAD_RODATA", self.rodata)?;
            region_alias(writer, "LOAD_DATA", self.data)?;
        }

        // Referenced in target code.
        writeln!(
            writer,
            "__flexram_config = {:#010X};",
            flexram_config(self.family, &self.flexram_layout)
        )?;
        // The target runtime looks at this value to predicate some pre-init instructions.
        // Could be helpful for binary identification, but it's an undocumented feature.
        writeln!(writer, "__imxrt_rt_v0.2 = {:#010X};", self.family.id(),)?;

        let link_x = include_bytes!("host/imxrt-link.x");
        writer.write_all(link_x)?;

        Ok(())
    }

    /// Implement i.MX RT specific sanity checks.
    ///
    /// This might not check everything! If the linker may detect a condition, we'll
    /// let the linker do that.
    fn check_configurations(&self) -> Result<(), String> {
        if self.family.flexram_bank_count() < self.flexram_layout.len() {
            return Err(format!(
                "Chip {:?} only has {} total FlexRAM banks. Cannot allocate {:?}, a total of {} banks",
                self.family,
                self.family.flexram_bank_count(),
                self.flexram_layout,
                self.flexram_layout.len(),
            ));
        }
        let ocram_count = layout_count_of(FlexRamKind::Ocram, &self.flexram_layout);
        if ocram_count < self.family.bootrom_ocram_banks() {
            return Err(format!(
                "Chip {:?} requires at least {} OCRAM banks for the bootloader ROM",
                self.family,
                self.family.bootrom_ocram_banks()
            ));
        }
        if let Some(flash_opts) = &self.flash_opts
            && !flash_opts.flexspi.supported_for_family(self.family)
        {
            return Err(format!(
                "Chip {:?} does not support {:?}",
                self.family, flash_opts.flexspi
            ));
        }

        fn prevent_flash(name: &str, memory: Memory) -> Result<(), String> {
            if memory == Memory::Flash {
                Err(format!("Section '{name}' cannot be placed in flash"))
            } else {
                Ok(())
            }
        }
        macro_rules! prevent_flash {
            ($sec:ident) => {
                prevent_flash(stringify!($sec), self.$sec)
            };
        }

        prevent_flash!(data)?;
        prevent_flash!(vectors)?;
        prevent_flash!(bss)?;
        prevent_flash!(uninit)?;
        prevent_flash!(stack)?;
        prevent_flash!(heap)?;

        Ok(())
    }
}

/// Write RAM-like memory blocks.
///
/// Skips a section if there's no FlexRAM block allocated. If a user references one
/// of this skipped sections, linking fails.
fn write_flexram_memories(
    output: &mut dyn Write,
    family: Family,
    flexram_layout: &[FlexRamKind],
) -> io::Result<()> {
    let itcm_count = layout_count_of(FlexRamKind::Itcm, flexram_layout);
    let dtcm_count = layout_count_of(FlexRamKind::Dtcm, flexram_layout);
    let ocram_count = layout_count_of(FlexRamKind::Ocram, flexram_layout);

    if itcm_count > 0 {
        let (itcm_start, itcm_size) = family.itcm_start_size(itcm_count);
        writeln!(
            output,
            "ITCM (RWX) : ORIGIN = {itcm_start:#X}, LENGTH = {itcm_size:#X}"
        )?;
    }
    if dtcm_count > 0 {
        writeln!(
            output,
            "DTCM (RWX) : ORIGIN = 0x20000000, LENGTH = {:#X}",
            dtcm_count * family.flexram_bank_size(),
        )?;
    }

    let ocram_size = ocram_count * family.flexram_bank_size() + family.dedicated_ocram_size();
    if ocram_size > 0 {
        writeln!(
            output,
            "OCRAM (RWX) : ORIGIN = {:#X}, LENGTH = {:#X}",
            family.ocram_start(),
            ocram_size,
        )?;
    }
    Ok(())
}

/// Generate a linker script MEMORY command that includes a FLASH block.
fn write_flash_memory_map(
    output: &mut dyn Write,
    family: Family,
    flash_opts: &FlashOpts,
    flexram_layout: &[FlexRamKind],
) -> io::Result<()> {
    writeln!(
        output,
        "/* Memory map for '{:?}' with custom flash length {}. */",
        family, flash_opts.size
    )?;
    writeln!(output, "MEMORY {{")?;
    writeln!(
        output,
        "FLASH (RX) : ORIGIN = {:#X}, LENGTH = {:#X}",
        flash_opts.flash_origin(family).expect("Already checked"),
        flash_opts.size
    )?;
    write_flexram_memories(output, family, flexram_layout)?;
    writeln!(output, "}}")?;
    writeln!(output, "__fcb_offset = {:#X};", family.fcb_offset())?;
    Ok(())
}

/// Generate a linker script MEMORY command that supports RAM execution.
///
/// It's like [`write_flash_memory_map`], but it doesn't include the flash
/// important tidbits.
fn write_ram_memory_map(
    output: &mut dyn Write,
    family: Family,
    flexram_layout: &[FlexRamKind],
) -> io::Result<()> {
    writeln!(
        output,
        "/* Memory map for '{family:?}' that executes from RAM. */",
    )?;
    writeln!(output, "MEMORY {{")?;
    write_flexram_memories(output, family, flexram_layout)?;
    writeln!(output, "}}")?;
    Ok(())
}

/// i.MX RT chip family.
///
/// Chip families are designed by reference manuals and produce categories.
/// Supply this to a [`RuntimeBuilder`] in order to check runtime configurations.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Imxrt1010,
    Imxrt1015,
    Imxrt1020,
    Imxrt1040,
    Imxrt1050,
    Imxrt1060,
    Imxrt1064,
    Imxrt1160,
    Imxrt1170,
    Imxrt1180,
}

/// Adding a new MCU? You'll probably need to update
/// these methods.
impl Family {
    /// Family identifier.
    ///
    /// These values may be stored in the image and observe by the runtime
    /// initialzation routine. Make sure these numbers are kept in sync with
    /// any hard-coded values.
    const fn id(self) -> u32 {
        match self {
            Family::Imxrt1010 => 0x1010,
            Family::Imxrt1015 => 0x1015,
            Family::Imxrt1020 => 0x1020,
            Family::Imxrt1040 => 0x1040,
            Family::Imxrt1050 => 0x1050,
            Family::Imxrt1060 => 0x1060,
            Family::Imxrt1064 => 0x1064,
            Family::Imxrt1160 => 0x1160,
            Family::Imxrt1170 => 0x1170,
            Family::Imxrt1180 => 0x1180,
        }
    }
    /// How many FlexRAM banks are available?
    pub const fn flexram_bank_count(self) -> usize {
        match self {
            Family::Imxrt1010 | Family::Imxrt1015 => 4,
            Family::Imxrt1020 => 8,
            Family::Imxrt1040 | Family::Imxrt1050 | Family::Imxrt1060 | Family::Imxrt1064 => 16,
            // No ECC support; treating all banks as equal.
            Family::Imxrt1160 | Family::Imxrt1170 => 16,
            Family::Imxrt1180 => 2,
        }
    }
    /// How large (bytes) is each FlexRAM bank?
    const fn flexram_bank_size(self) -> usize {
        match self {
            Family::Imxrt1010
            | Family::Imxrt1015
            | Family::Imxrt1020
            | Family::Imxrt1040
            | Family::Imxrt1050
            | Family::Imxrt1060
            | Family::Imxrt1064
            | Family::Imxrt1160
            | Family::Imxrt1170 => 32 * 1024,
            Family::Imxrt1180 => 128 * 1024,
        }
    }
    /// How many OCRAM banks does the boot ROM need?
    const fn bootrom_ocram_banks(self) -> usize {
        match self {
            Family::Imxrt1010
            | Family::Imxrt1015
            | Family::Imxrt1020
            | Family::Imxrt1040
            | Family::Imxrt1050 => 1,
            // 9.5.1. memory maps point at OCRAM2.
            Family::Imxrt1060 | Family::Imxrt1064 => 0,
            // Boot ROM uses dedicated OCRAM1.
            Family::Imxrt1160 | Family::Imxrt1170 | Family::Imxrt1180 => 0,
        }
    }
    /// Where's the FlexSPI configuration bank located?
    fn fcb_offset(self) -> usize {
        match self {
            Family::Imxrt1010 | Family::Imxrt1160 | Family::Imxrt1170 | Family::Imxrt1180 => 0x400,
            Family::Imxrt1015
            | Family::Imxrt1020
            | Family::Imxrt1040
            | Family::Imxrt1050
            | Family::Imxrt1060
            | Family::Imxrt1064 => 0x000,
        }
    }

    /// Where does the OCRAM region begin?
    ///
    /// This includes dedicated any OCRAM regions, if any exist for the chip.
    fn ocram_start(self) -> u32 {
        match self {
            // 256 KiB offset from the OCRAM M4 backdoor.
            Family::Imxrt1170 => 0x2024_0000,
            // Using the alias regions, assuming ECC is disabled.
            // The two alias regions, plus the ECC region, provide
            // the *contiguous* 256 KiB of dedicated OCRAM.
            Family::Imxrt1160 => 0x2034_0000,
            // Skip the first 16 KiB, "cannot be safely used by application images".
            Family::Imxrt1180 => 0x2048_4000,
            // Either starts the FlexRAM OCRAM banks, or the
            // dedicated OCRAM regions (for supported devices).
            Family::Imxrt1010
            | Family::Imxrt1015
            | Family::Imxrt1020
            | Family::Imxrt1040
            | Family::Imxrt1050
            | Family::Imxrt1060
            | Family::Imxrt1064 => 0x2020_0000,
        }
    }

    /// What's the size, in bytes, of the dedicated OCRAM section?
    ///
    /// This isn't supported by all chips.
    const fn dedicated_ocram_size(self) -> usize {
        match self {
            Family::Imxrt1010
            | Family::Imxrt1015
            | Family::Imxrt1020
            | Family::Imxrt1040
            | Family::Imxrt1050 => 0,
            Family::Imxrt1060 | Family::Imxrt1064 => 512 * 1024,
            // - Two dedicated OCRAMs
            // - One FlexRAM OCRAM EC region that's OCRAM when ECC is disabled.
            Family::Imxrt1160 => (2 * 64 + 128) * 1024,
            // - Two dedicated OCRAMs
            // - Two dedicated OCRAM ECC regions that aren't used for ECC
            // - One FlexRAM OCRAM ECC region that's strictly OCRAM, without ECC
            Family::Imxrt1170 => (2 * 512 + 2 * 64 + 128) * 1024,
            // OCRAM1 (512k), OCRAM2 (256k), 16k reserved as a ROM patch area
            Family::Imxrt1180 => (512 + 256 - 16) * 1024,
        }
    }

    /// Returns the default FlexRAM bank allocations for this chip.
    ///
    /// The default values represent the all-zero fuse values. However,
    /// the layout is an implementation detail.
    pub fn default_flexram_banks(self) -> FlexRamBanks {
        match self {
            Family::Imxrt1010 | Family::Imxrt1015 => FlexRamBanks {
                ocram: 2,
                itcm: 1,
                dtcm: 1,
            },
            Family::Imxrt1020 => FlexRamBanks {
                ocram: 4,
                itcm: 2,
                dtcm: 2,
            },
            Family::Imxrt1040 | Family::Imxrt1050 | Family::Imxrt1060 | Family::Imxrt1064 => {
                FlexRamBanks {
                    ocram: 8,
                    itcm: 4,
                    dtcm: 4,
                }
            }
            Family::Imxrt1160 | Family::Imxrt1170 => FlexRamBanks {
                ocram: 0,
                itcm: 8,
                dtcm: 8,
            },
            Family::Imxrt1180 => FlexRamBanks {
                ocram: 0,
                itcm: 1,
                dtcm: 1,
            },
        }
    }

    /// Returns the default FlexRAM bank layout for this chip.
    ///
    /// The default values represent the all-zero fuse values.
    /// See AN12077 for details.
    pub fn default_flexram_layout(self) -> Vec<FlexRamKind> {
        match self {
            Family::Imxrt1010 | Family::Imxrt1015 => vec![
                FlexRamKind::Ocram,
                FlexRamKind::Ocram,
                FlexRamKind::Dtcm,
                FlexRamKind::Itcm,
            ],
            Family::Imxrt1020 => vec![
                FlexRamKind::Ocram,
                FlexRamKind::Ocram,
                FlexRamKind::Dtcm,
                FlexRamKind::Dtcm,
                FlexRamKind::Itcm,
                FlexRamKind::Itcm,
                FlexRamKind::Ocram,
                FlexRamKind::Ocram,
            ],
            // 1040 layout described in table 22-9 of the RM.
            // It's not convered in AN12077.
            Family::Imxrt1040 | Family::Imxrt1050 | Family::Imxrt1060 | Family::Imxrt1064 => vec![
                FlexRamKind::Ocram,
                FlexRamKind::Ocram,
                FlexRamKind::Ocram,
                FlexRamKind::Ocram,
                FlexRamKind::Dtcm,
                FlexRamKind::Dtcm,
                FlexRamKind::Itcm,
                FlexRamKind::Itcm,
                FlexRamKind::Itcm,
                FlexRamKind::Itcm,
                FlexRamKind::Dtcm,
                FlexRamKind::Dtcm,
                FlexRamKind::Ocram,
                FlexRamKind::Ocram,
                FlexRamKind::Ocram,
                FlexRamKind::Ocram,
            ],
            Family::Imxrt1160 | Family::Imxrt1170 => vec![
                FlexRamKind::Dtcm,
                FlexRamKind::Dtcm,
                FlexRamKind::Dtcm,
                FlexRamKind::Dtcm,
                FlexRamKind::Itcm,
                FlexRamKind::Itcm,
                FlexRamKind::Itcm,
                FlexRamKind::Itcm,
                FlexRamKind::Dtcm,
                FlexRamKind::Dtcm,
                FlexRamKind::Dtcm,
                FlexRamKind::Dtcm,
                FlexRamKind::Itcm,
                FlexRamKind::Itcm,
                FlexRamKind::Itcm,
                FlexRamKind::Itcm,
            ],
            // Layout doesn't matter; we only have three
            // configurations.
            Family::Imxrt1180 => vec![FlexRamKind::Itcm, FlexRamKind::Dtcm],
        }
    }

    /// Returns the start and size of the ITCM memory region.
    const fn itcm_start_size(self, itcm_banks: usize) -> (usize, usize) {
        let mut itcm_size = itcm_banks * self.flexram_bank_size();
        let itcm_start = match self {
            Family::Imxrt1010
            | Family::Imxrt1015
            | Family::Imxrt1020
            | Family::Imxrt1040
            | Family::Imxrt1050
            | Family::Imxrt1060
            | Family::Imxrt1064
            | Family::Imxrt1160
            | Family::Imxrt1170 => {
                // Establish a reservation for null pointers.
                // Note that this reservation is the minimum
                // size of an MPU region.
                itcm_size = itcm_size.saturating_sub(32);
                32
            }
            Family::Imxrt1180 => 0x10000000 - itcm_size,
        };
        (itcm_start, itcm_size)
    }
}

/// If you're adding a new MCU family, you probably
/// don't need to change these methods.
impl Family {
    /// Returns the starting address for the given `flexspi` instance.
    ///
    /// If a FlexSPI instance isn't available for this family, the return
    /// is `None`. Otherwise, the return is the starting address in the
    /// MCU's memory map.
    ///
    /// ```
    /// use imxrt_rt::{Family::*, FlexSpi::*};
    ///
    /// assert_eq!(Imxrt1060.flexspi_start_addr(FlexSpi1), Some(0x6000_0000));
    /// assert!(Imxrt1010.flexspi_start_addr(FlexSpi2).is_none());
    /// ```
    pub fn flexspi_start_addr(self, flexspi: FlexSpi) -> Option<u32> {
        flexspi.start_address(self)
    }
}

/// FlexRAM bank allocations.
///
/// Depending on your device, you may need a non-zero number of
/// OCRAM banks to support the boot ROM. Consult your processor's
/// reference manual for more information.
///
/// You should keep the sum of all banks below or equal to the
/// total number of banks supported by your device. Unallocated memory
/// banks are disabled.
///
/// Banks are typically 32KiB large.
///
/// If you need to control the _layout_, or assignment, of FlexRAM
/// banks, you should define your own collection of [`FlexRamKind`]
/// and use [`flexram_layout`](RuntimeBuilder::flexram_layout) to
/// set the layout. If you use this to select the bank counts, the
/// builder applies an unspecified layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlexRamBanks {
    /// How many banks are allocated for OCRAM?
    ///
    /// This may need to be non-zero to support the boot ROM.
    /// Consult your reference manual.
    ///
    /// Note: these are FlexRAM OCRAM banks. Do not include any banks
    /// that would represent dedicated OCRAM; the runtime implementation
    /// allocates those automatically. In fact, if your chip includes
    /// dedicated OCRAM, you may set this to zero in order to maximize
    /// DTCM and ITCM utilization.
    pub ocram: usize,
    /// How many banks are allocated for ITCM?
    pub itcm: usize,
    /// How many banks are allocated for DTCM?
    pub dtcm: usize,
}

impl FlexRamBanks {
    /// Convert the banks into some kind of FlexRAM bank layout.
    fn to_flexram_layout(self) -> Vec<FlexRamKind> {
        let mut layout = Vec::with_capacity(self.ocram + self.dtcm + self.itcm);
        for _ in 0..self.ocram {
            layout.push(FlexRamKind::Ocram);
        }
        for _ in 0..self.dtcm {
            layout.push(FlexRamKind::Dtcm);
        }
        for _ in 0..self.itcm {
            layout.push(FlexRamKind::Itcm);
        }
        layout
    }
}

/// Describes how a FlexRAM bank is being used.
///
/// These are the elements of a "layout," usually
/// represented by `&[FlexRamKind]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum FlexRamKind {
    /// It's not used at all.
    Unused = 0,
    /// This bank is an OCRAM bank.
    ///
    /// Remember that these are FlexRAM banks, not dedicated
    /// OCRAM banks.
    Ocram = 1,
    /// This is a DTCM bank.
    Dtcm = 2,
    /// This is an ITCM bank.
    Itcm = 3,
}

/// Count how may RAM kinds there are in this layout.
fn layout_count_of(kind: FlexRamKind, layout: &[FlexRamKind]) -> usize {
    layout.iter().filter(|k| **k == kind).count()
}

/// Produce the `u32` describing the FlexRAM configuration
/// for the MCU.
fn flexram_config(family: Family, layout: &[FlexRamKind]) -> u32 {
    assert!(
        layout.len() <= family.flexram_bank_count(),
        "FlexRAM layout contains too many banks for {family:?}"
    );

    if family == Family::Imxrt1180 {
        let itcm_count = layout_count_of(FlexRamKind::Itcm, layout);
        let dtcm_count = layout_count_of(FlexRamKind::Dtcm, layout);
        let ocram_count = layout_count_of(FlexRamKind::Ocram, layout);
        match (itcm_count, dtcm_count, ocram_count) {
            (1, 1, 0) => 0b00_u32,
            (2, 0, 0) => 0b10,
            (0, 2, 0) => 0b01,
            _ => panic!("Unsupported FlexRAM configuration"),
        }
    } else {
        let mut mask = 0;
        let mut shift = 0;
        for kind in layout {
            mask |= (*kind as u32) << shift;
            shift += 2;
        }
        mask
    }
}

#[cfg(test)]
mod tests {
    use crate::Memory;

    use super::{Family, FlexRamBanks, RuntimeBuilder};
    use std::{error, io};

    const MOST_FAMILIES: &[Family] = &[
        Family::Imxrt1010,
        Family::Imxrt1015,
        Family::Imxrt1020,
        Family::Imxrt1040,
        Family::Imxrt1050,
        Family::Imxrt1060,
        Family::Imxrt1064,
        Family::Imxrt1170,
        // Imxrt1180 wasn't ever tested here.
    ];
    type Error = Box<dyn error::Error>;

    #[test]
    fn flexram_config() {
        /// Testing table of banks and expected configuration mask.
        #[allow(clippy::unusual_byte_groupings)] // Spacing delimits ITCM / DTCM / OCRAM banks.
        const TABLE: &[(FlexRamBanks, u32)] = &[
            (
                FlexRamBanks {
                    ocram: 16,
                    dtcm: 0,
                    itcm: 0,
                },
                0x55555555,
            ),
            (
                FlexRamBanks {
                    ocram: 0,
                    dtcm: 16,
                    itcm: 0,
                },
                0xAAAAAAAA,
            ),
            (
                FlexRamBanks {
                    ocram: 0,
                    dtcm: 0,
                    itcm: 16,
                },
                0xFFFFFFFF,
            ),
            (
                FlexRamBanks {
                    ocram: 0,
                    dtcm: 0,
                    itcm: 0,
                },
                0,
            ),
            (
                FlexRamBanks {
                    ocram: 1,
                    dtcm: 1,
                    itcm: 1,
                },
                0b11_10_01,
            ),
            (
                FlexRamBanks {
                    ocram: 3,
                    dtcm: 3,
                    itcm: 3,
                },
                0b111111_101010_010101,
            ),
            (
                FlexRamBanks {
                    ocram: 5,
                    dtcm: 5,
                    itcm: 5,
                },
                0b1111111111_1010101010_0101010101,
            ),
            (
                FlexRamBanks {
                    ocram: 1,
                    dtcm: 1,
                    itcm: 14,
                },
                0b1111111111111111111111111111_10_01,
            ),
            (
                FlexRamBanks {
                    ocram: 1,
                    dtcm: 14,
                    itcm: 1,
                },
                0b11_1010101010101010101010101010_01,
            ),
            (
                FlexRamBanks {
                    ocram: 14,
                    dtcm: 1,
                    itcm: 1,
                },
                0b11_10_0101010101010101010101010101,
            ),
        ];

        for (banks, expected) in TABLE {
            // Select a family that has all 16 banks available.
            let actual = super::flexram_config(Family::Imxrt1170, &banks.to_flexram_layout());
            assert!(
                actual == *expected,
                "\nActual:   {actual:#034b}\nExpected: {expected:#034b}\nBanks: {banks:?}"
            );
        }
    }

    #[test]
    fn runtime_builder_default_from_flexspi() -> Result<(), Error> {
        for family in MOST_FAMILIES {
            RuntimeBuilder::from_flexspi(*family, 16 * 1024 * 1024)
                .write_linker_script(&mut io::sink())?;
        }
        Ok(())
    }

    /// Strange but currently allowed.
    #[test]
    fn runtime_builder_from_flexspi_no_flash() -> Result<(), Error> {
        RuntimeBuilder::from_flexspi(Family::Imxrt1060, 0).write_linker_script(&mut io::sink())
    }

    #[test]
    fn runtime_builder_too_many_flexram_banks() {
        let banks = FlexRamBanks {
            itcm: 32,
            dtcm: 32,
            ocram: 32,
        };
        for family in MOST_FAMILIES {
            let res = RuntimeBuilder::from_flexspi(*family, 16 * 1024)
                .flexram_banks(banks)
                .write_linker_script(&mut io::sink());
            assert!(res.is_err(), "{family:?}");
        }
    }

    #[test]
    fn runtime_builder_invalid_flash_section() {
        type Placer = fn(&mut RuntimeBuilder) -> &mut RuntimeBuilder;
        macro_rules! placement {
            ($section:ident) => {
                (|bldr| bldr.$section(Memory::Flash), stringify!($section))
            };
        }
        let placements: &[(Placer, &'static str)] = &[
            placement!(data),
            placement!(vectors),
            placement!(bss),
            placement!(uninit),
            placement!(stack),
            placement!(heap),
        ];

        for family in MOST_FAMILIES {
            for placement in placements {
                let mut bldr = RuntimeBuilder::from_flexspi(*family, 16 * 1024);
                placement.0(&mut bldr);
                let res = bldr.write_linker_script(&mut io::sink());
                assert!(res.is_err(), "{:?}, section: {}", family, placement.1);
            }
        }
    }

    #[test]
    fn itcm_start_size() {
        // Most parts have an ITCM that could touch address 0.
        // However, the implementation reserves an MPU region
        // at address 0.
        for family in MOST_FAMILIES {
            for itcm_banks in 0..=family.flexram_bank_count() {
                let (start, size) = family.itcm_start_size(itcm_banks);
                assert_eq!(start, 32);
                assert_eq!(
                    size,
                    (family.flexram_bank_size() * itcm_banks).saturating_sub(32)
                );
            }
        }

        // The 1180's ITCM never touches address 0 when the ITCM banks
        // are properly configured.
        let family = Family::Imxrt1180;
        for itcm_banks in 0..=family.flexram_bank_count() {
            let (start, size) = family.itcm_start_size(itcm_banks);
            assert_ne!(start, 0);
            assert_eq!(size, family.flexram_bank_size() * itcm_banks);
        }
    }

    #[test]
    fn default_flexram_layouts() {
        let cases = [
            (Family::Imxrt1010, 0b11100101),
            (Family::Imxrt1015, 0b11100101),
            (Family::Imxrt1020, 0b0101111110100101),
            (Family::Imxrt1040, 0b01010101101011111111101001010101),
            (Family::Imxrt1050, 0b01010101101011111111101001010101),
            (Family::Imxrt1060, 0b01010101101011111111101001010101),
            (Family::Imxrt1064, 0b01010101101011111111101001010101),
            (Family::Imxrt1160, 0b11111111101010101111111110101010),
            (Family::Imxrt1170, 0b11111111101010101111111110101010),
            (Family::Imxrt1180, 0b00),
        ];
        for (family, expected) in cases {
            let layout = family.default_flexram_layout();
            let actual = super::flexram_config(family, &layout);
            assert_eq!(
                actual, expected,
                "{family:?} {actual:#010X} {expected:#010X}"
            );
        }
    }
}
