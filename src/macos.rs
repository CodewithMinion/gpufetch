//! macOS GPU detection via system_profiler

use crate::error::{Result, GpufetchError};
use crate::global::{info, warn};
use crate::gpu::{GpuInfo, TopologyApple, TopologyIntel, Uarch, Vendor};
use std::process::Command;

struct DisplayBlock {
    name: String,
    chipset: String,
    device_type: String,
    vram_mb: Option<u32>,
    gpu_cores: Option<u32>,
}

/// Lists GPUs reported by system_profiler
pub fn list_gpus() -> Result<()> {
    info("Listing GPUs via system_profiler...");
    for gpu in detect_display_blocks()? {
        if gpu.is_gpu() {
            println!("{} ({})", gpu.chipset, gpu.name);
        }
    }
    Ok(())
}

/// Returns GPU information for the given index
pub fn get_gpu_info(idx: i32) -> Result<Option<GpuInfo>> {
    let gpus: Vec<GpuInfo> = detect_display_blocks()?
        .into_iter()
        .filter(|d| d.is_gpu())
        .filter_map(|d| build_gpu_info(&d).ok().flatten())
        .collect();

    if idx < 0 {
        return Ok(gpus.into_iter().next());
    }

    Ok(gpus.get(idx as usize).cloned())
}

fn detect_display_blocks() -> Result<Vec<DisplayBlock>> {
    let output = Command::new("system_profiler")
        .args(["SPDisplaysDataType"])
        .output()
        .map_err(GpufetchError::CommandFailed)?;

    if !output.status.success() {
        warn("system_profiler returned non-zero exit code");
        return Ok(Vec::new());
    }

    Ok(parse_system_profiler(&String::from_utf8_lossy(&output.stdout)))
}

fn parse_system_profiler(text: &str) -> Vec<DisplayBlock> {
    let mut blocks = Vec::new();
    let mut current: Option<DisplayBlock> = None;

    for line in text.lines() {
        // Top-level display section: "    Apple M1:"
        if line.starts_with("    ") && !line.starts_with("      ") {
            if let Some(block) = current.take() {
                blocks.push(block);
            }
            let header = line[4..].trim_end_matches(':').trim();
            if header.eq_ignore_ascii_case("Graphics/Displays") {
                continue;
            }
            current = Some(DisplayBlock {
                name: header.to_string(),
                chipset: String::new(),
                device_type: String::new(),
                vram_mb: None,
                gpu_cores: None,
            });
            continue;
        }

        let Some(block) = current.as_mut() else {
            continue;
        };

        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("Chipset Model:") {
            block.chipset = value.trim().to_string();
        } else if let Some(value) = trimmed.strip_prefix("Type:") {
            block.device_type = value.trim().to_string();
        } else if let Some(value) = trimmed.strip_prefix("VRAM (Dynamic, Max):") {
            block.vram_mb = parse_memory_mb(value);
        } else if let Some(value) = trimmed.strip_prefix("VRAM (Total):") {
            block.vram_mb = parse_memory_mb(value);
        } else if let Some(value) = trimmed.strip_prefix("Total Number of Cores:") {
            block.gpu_cores = value.trim().parse().ok();
        }
    }

    if let Some(block) = current {
        blocks.push(block);
    }

    blocks
}

fn parse_memory_mb(value: &str) -> Option<u32> {
    value
        .split_whitespace()
        .next()
        .and_then(|n| n.parse().ok())
}

impl DisplayBlock {
    fn is_gpu(&self) -> bool {
        let name = self.display_name();
        if name.is_empty() {
            return false;
        }
        if self.device_type.eq_ignore_ascii_case("GPU") {
            return true;
        }
        // Built-in Apple/Intel/AMD GPUs often omit Type on older macOS versions
        !name.contains("Color LCD")
            && !name.contains("Display")
            && (name.contains("Apple")
                || name.contains("Intel")
                || name.contains("AMD")
                || name.contains("Radeon")
                || name.contains("NVIDIA")
                || name.contains("GeForce"))
    }

    fn display_name(&self) -> String {
        if !self.chipset.is_empty() {
            self.chipset.clone()
        } else {
            self.name.clone()
        }
    }
}

fn build_gpu_info(block: &DisplayBlock) -> Result<Option<GpuInfo>> {
    let name = block.display_name();
    info(&format!("Found macOS GPU: {name}"));

    if name.contains("Apple") {
        return Ok(Some(build_apple_gpu(&name, block.gpu_cores)));
    }
    if name.contains("Intel") {
        return Ok(Some(build_intel_gpu_from_name(&name, block.vram_mb)));
    }
    if name.contains("AMD") || name.contains("Radeon") {
        return Ok(Some(build_amd_gpu_from_name(&name)));
    }
    if name.contains("NVIDIA") || name.contains("GeForce") {
        return Ok(Some(build_nvidia_gpu_from_name(&name)));
    }

    warn(&format!("Unknown macOS GPU: {name}"));
    Ok(None)
}

fn build_apple_gpu(name: &str, cores: Option<u32>) -> GpuInfo {
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

fn build_intel_gpu_from_name(name: &str, vram_mb: Option<u32>) -> GpuInfo {
    let (arch_name, gen, gt, eu) = match_intel_name(name);
    let freq_mhz = get_intel_freq_from_name(name);

    let topology = TopologyIntel {
        subslices: if gt > 0 { (gt * 2) as i32 } else { -1 },
        eu_per_subslice: if eu > 0 && gt > 0 {
            eu as i32 / gt as i32 / 2
        } else {
            -1
        },
        gt: gt as i32,
    };

    let arch = Uarch {
        name: arch_name.to_string(),
        process_nm: get_intel_process_node(gen),
        chip: format!("Gen{gen}"),
        compute_capability: 0,
        llvm_target: 0,
        gt: gt as i32,
        eu: eu as i32,
    };

    let peak_perf = if eu > 0 && freq_mhz > 0 {
        Some((freq_mhz as i64) * 1_000_000 * (eu as i64) * 2)
    } else {
        None
    };

    let mut gpu = GpuInfo::new(Vendor::Intel, name.to_string(), arch, freq_mhz);
    gpu.topology_intel = Some(topology);
    gpu.peak_performance = peak_perf;

    if let Some(vram) = vram_mb {
        gpu.memory = Some(crate::gpu::Memory {
            size_bytes: (vram as u64) * 1024 * 1024,
            freq_mhz: 0,
            bus_width: 128,
            memtype: crate::gpu::MemoryType::Unknown,
        });
    }

    gpu
}

fn match_intel_name(name: &str) -> (&'static str, u8, u8, u16) {
    let lower = name.to_lowercase();
    if lower.contains("iris xe") {
        ("Xe", 12, 2, 96)
    } else if lower.contains("iris plus") {
        ("Ice Lake", 11, 2, 48)
    } else if lower.contains("iris") {
        ("Broadwell", 8, 2, 24)
    } else if lower.contains("uhd 630") {
        ("Coffee Lake", 9, 2, 24)
    } else if lower.contains("hd graphics 6000") || lower.contains("hd 6000") {
        ("Broadwell", 8, 2, 24)
    } else if lower.contains("hd graphics 5000") || lower.contains("hd 5000") {
        ("Haswell", 7, 2, 20)
    } else if lower.contains("hd graphics 4000") || lower.contains("hd 4000") {
        ("Ivy Bridge", 7, 2, 16)
    } else if lower.contains("hd graphics 3000") || lower.contains("hd 3000") {
        ("Sandy Bridge", 6, 2, 12)
    } else {
        ("Intel Graphics", 9, 2, 24)
    }
}

fn get_intel_freq_from_name(name: &str) -> u32 {
    let lower = name.to_lowercase();
    if lower.contains("iris xe") {
        1450
    } else if lower.contains("iris plus") {
        1250
    } else if lower.contains("uhd") {
        1200
    } else {
        1150
    }
}

fn get_intel_process_node(gen: u8) -> i32 {
    match gen {
        6..=7 => 22,
        8..=9 => 14,
        10..=11 => 10,
        12..=13 => 7,
        _ => -1,
    }
}

fn build_amd_gpu_from_name(name: &str) -> GpuInfo {
    let arch = Uarch {
        name: "RDNA".to_string(),
        process_nm: 7,
        chip: "Apple Silicon dGPU".to_string(),
        compute_capability: 0,
        llvm_target: 7,
        gt: 0,
        eu: 0,
    };
    GpuInfo::new(Vendor::Amd, name.to_string(), arch, 1900)
}

fn build_nvidia_gpu_from_name(name: &str) -> GpuInfo {
    let arch = Uarch {
        name: "Unknown".to_string(),
        process_nm: -1,
        chip: name.to_string(),
        compute_capability: 0,
        llvm_target: 0,
        gt: 0,
        eu: 0,
    };
    GpuInfo::new(Vendor::Nvidia, name.to_string(), arch, 1500)
}
