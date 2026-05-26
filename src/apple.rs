//! Apple GPU helpers (macOS + Linux on Apple hardware, e.g. Asahi)

use crate::common;
use crate::error::Result;
use crate::gpu::{GpuInfo, TopologyApple, Uarch, Vendor};
use std::fs;
use std::path::Path;

/// Build `GpuInfo` for an Apple Silicon / Apple GPU by marketing name.
pub fn build_apple_gpu(name: &str, cores: Option<u32>) -> GpuInfo {
    let (arch_name, process_nm, chip, gpu_cores, freq_mhz) = match_apple_gpu(name, cores);

    let arch = Uarch {
        name: arch_name.to_string(),
        process_nm,
        chip: chip.to_string(),
        compute_capability: 0,
        llvm_target: 0,
        gt: 0,
        eu: 0,
    };

    let mut gpu = GpuInfo::new(Vendor::Apple, name.to_string(), arch, freq_mhz);
    gpu.topology_apple = Some(TopologyApple { gpu_cores });
    gpu
}

fn match_apple_gpu(name: &str, cores: Option<u32>) -> (&'static str, i32, &'static str, u32, u32) {
    let lower = name.to_lowercase();
    let cores = cores.unwrap_or(0);

    if lower.contains("m4") {
        ("M4 GPU", 3, "M4", cores.max(10), 1500)
    } else if lower.contains("m3") {
        ("M3 GPU", 3, "M3", cores.max(10), 1380)
    } else if lower.contains("m2") {
        ("M2 GPU", 5, "M2", cores.max(10), 1398)
    } else if lower.contains("m1") {
        ("M1 GPU", 5, "M1", cores.max(8), 1278)
    } else {
        ("Apple GPU", 5, "Apple Silicon", cores.max(8), 1300)
    }
}

/// Derive a display name like `Apple M1` from a device-tree model string.
pub fn gpu_name_from_machine_model(model: &str) -> String {
    let lower = model.to_lowercase();
    if lower.contains("m4") {
        return "Apple M4".to_string();
    }
    if lower.contains("m3") {
        return "Apple M3".to_string();
    }
    if lower.contains("m2") {
        return "Apple M2".to_string();
    }
    if lower.contains("m1") {
        return "Apple M1".to_string();
    }
    format!("Apple GPU ({model})")
}

#[cfg(target_os = "linux")]
pub mod linux {
    use super::*;
    use crate::global::info;

    pub fn list_gpus() -> Result<()> {
        if let Some((name, _)) = detect_apple_gpu() {
            println!("{name} (Apple, device-tree/PCI)");
        }
        Ok(())
    }

    pub fn get_gpu_info(idx: i32) -> Result<Option<GpuInfo>> {
        if idx > 0 {
            return Ok(None);
        }
        let Some((name, cores)) = detect_apple_gpu() else {
            return Ok(None);
        };
        info(&format!("Found Apple GPU on Linux: {name}"));
        Ok(Some(build_apple_gpu(&name, cores)))
    }

    fn detect_apple_gpu() -> Option<(String, Option<u32>)> {
        if let Some(model) = read_devicetree_model() {
            if is_apple_mac_model(&model) {
                let name = gpu_name_from_machine_model(&model);
                return Some((name, None));
            }
        }

        if let Some(gpu) = detect_apple_drm_sysfs() {
            return Some(gpu);
        }

        detect_apple_lspci()
    }

    fn read_devicetree_model() -> Option<String> {
        for path in [
            "/sys/firmware/devicetree/base/model",
            "/proc/device-tree/model",
        ] {
            if let Ok(bytes) = fs::read(path) {
                let model: String = bytes
                    .iter()
                    .copied()
                    .filter(|b| *b != 0)
                    .map(|b| b as char)
                    .collect();
                let model = model.trim().to_string();
                if !model.is_empty() {
                    return Some(model);
                }
            }
        }
        None
    }

    fn is_apple_mac_model(model: &str) -> bool {
        model.contains("MacBook")
            || model.contains("Mac mini")
            || model.contains("Mac Studio")
            || model.contains("Mac Pro")
            || model.contains("iMac")
            || model.starts_with("Mac")
    }

    fn detect_apple_drm_sysfs() -> Option<(String, Option<u32>)> {
        let drm_dir = fs::read_dir("/sys/class/drm").ok()?;
        for entry in drm_dir.flatten() {
            let card = entry.path();
            let vendor_path = card.join("device/vendor");
            let Ok(vendor) = fs::read_to_string(&vendor_path) else {
                continue;
            };
            let vendor = vendor.trim().to_lowercase();
            if vendor == "0x106b" || vendor == "106b" {
                let name = read_drm_device_name(&card).unwrap_or_else(|| "Apple GPU".to_string());
                return Some((name, None));
            }
        }
        None
    }

    fn read_drm_device_name(card: &Path) -> Option<String> {
        let uevent = fs::read_to_string(card.join("device/uevent")).ok()?;
        for line in uevent.lines() {
            if let Some(value) = line.strip_prefix("PCI_NAME=") {
                return Some(value.to_string());
            }
            if let Some(value) = line.strip_prefix("DRIVER=") {
                if value.contains("apple") || value.contains("asahi") {
                    return Some("Apple GPU".to_string());
                }
            }
        }
        None
    }

    fn detect_apple_lspci() -> Option<(String, Option<u32>)> {
        let output = common::try_output("lspci", &["-nn"])?;
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            let lower = line.to_lowercase();
            if lower.contains("106b:") || lower.contains("apple inc") {
                let name = line
                    .split_once(':')
                    .map(|(_, rest)| rest.trim())
                    .unwrap_or(line)
                    .split('[')
                    .next()
                    .unwrap_or("Apple GPU")
                    .trim()
                    .to_string();
                let display = if name.is_empty() {
                    "Apple GPU".to_string()
                } else {
                    gpu_name_from_machine_model(&name)
                };
                return Some((display, None));
            }
        }
        None
    }
}
