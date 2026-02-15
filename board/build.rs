use std::{collections::HashSet, env};

fn extract_features() -> HashSet<String> {
    env::vars()
        .map(|(k, _)| k)
        .flat_map(|feat| feat.strip_prefix("CARGO_FEATURE_").map(str::to_lowercase))
        .collect()
}

/// Creates a runtime for a particular family, adjusting whether the image is expected to be boot
/// based on provided features.
fn create_runtime(family: imxrt_rt::Family, flash_size: usize) -> imxrt_rt::RuntimeBuilder {
    if cfg!(feature = "nonboot") | cfg!(feature = "secondary") {
        let mut bldr = imxrt_rt::RuntimeBuilder::in_flash(family, flash_size, 256 * 1024);
        bldr.boot_header(cfg!(feature = "secondary"));
        bldr
    } else {
        imxrt_rt::RuntimeBuilder::from_flexspi(family, flash_size)
    }
}

/// Configures the runtime for a variety of boards.
///
/// Note that some automated tests may check these runtimes. Feel free to change
/// values and observe how they might affect the tests.
fn main() {
    let features = extract_features();
    for feature in features {
        match feature.as_str() {
            "teensy4" => {
                imxrt_rt::RuntimeBuilder::from_flexspi(imxrt_rt::Family::Imxrt1060, 1984 * 1024)
                    .flexram_banks(imxrt_rt::FlexRamBanks {
                        ocram: 0,
                        dtcm: 12,
                        itcm: 4,
                    })
                    .heap_size(1024)
                    .text(imxrt_rt::Memory::Flash)
                    .rodata(imxrt_rt::Memory::Dtcm)
                    .data(imxrt_rt::Memory::Dtcm)
                    .bss(imxrt_rt::Memory::Dtcm)
                    .uninit(imxrt_rt::Memory::Dtcm)
                    .stack_size_env_override("THIS_WONT_BE_CONSIDERED")
                    .stack_size_env_override("BOARD_STACK")
                    .heap_size_env_override("BOARD_HEAP")
                    .build()
                    .unwrap()
            }
            "imxrt1010evk" => imxrt_rt::RuntimeBuilder::from_flexspi(
                imxrt_rt::Family::Imxrt1010,
                16 * 1024 * 1024,
            )
            .heap_size(1024)
            .rodata(imxrt_rt::Memory::Flash)
            .stack_size_env_override("BOARD_STACK")
            .heap_size_env_override("BOARD_HEAP")
            .build()
            .unwrap(),
            "imxrt1170evk_cm7" => create_runtime(imxrt_rt::Family::Imxrt1170, 256 * 1024)
                .rodata(imxrt_rt::Memory::Dtcm)
                .stack_size_env_override("BOARD_STACK")
                .heap_size_env_override("BOARD_HEAP")
                .build()
                .unwrap(),
            "imxrt1010evk_ram" => {
                imxrt_rt::RuntimeBuilder::from_ram(imxrt_rt::Family::Imxrt1010)
                    .heap_size(1024)
                    .build()
                    .unwrap();
                println!("cargo:rustc-cfg=from_ram");
            }
            _ => continue,
        }
        break;
    }
}
