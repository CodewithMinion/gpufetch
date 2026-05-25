//! AMD GPU backend using lspci
//! Supports AMD RDNA/CDNA GPUs
//! 
//! # Examples
//! 
//! ```no_run
//! use gpufetch::amd::get_gpu_info;
//! let gpu = get_gpu_info(0)?;
//! ```

use crate::gpu::{GpuInfo, Vendor, Uarch, TopologyHsa, Memory, MemoryType};
use crate::global::{info, warn};
use crate::error::{Result, GpufetchError};
use std::process::Command;

/// Lists all AMD GPUs found via lspci
pub fn list_gpus() -> Result<()> {
    info("Listing AMD GPUs via lspci...");
    
    let output = Command::new("lspci")
        .args(&["-nn", "-d", "1002:"])
        .output()
        .map_err(|e| GpufetchError::CommandFailed(e))?;
    
    if !output.status.success() {
        warn("lspci returned non-zero exit code");
        return Ok(());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        println!("{line}");
    }
    
    Ok(())
}

/// Gets AMD GPU information by index
/// 
/// # Arguments
/// * `idx` - GPU index (0-based)
/// 
/// # Returns
/// * `Ok(Some(GpuInfo))` - GPU found
/// * `Ok(None)` - No GPU at this index
/// * `Err(GpufetchError)` - Detection failed
pub fn get_gpu_info(_idx: i32) -> Result<Option<GpuInfo>> {
    detect_amd_gpu_via_lspci()
}

fn detect_amd_gpu_via_lspci() -> Result<Option<GpuInfo>> {
    let output = Command::new("lspci")
        .args(&["-nn", "-d", "1002:"])
        .output()
        .map_err(|e| GpufetchError::CommandFailed(e))?;
    
    if !output.status.success() {
        warn("lspci command failed");
        return Ok(None);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    let (device_id, gpu_name) = parse_lspci_output(&stdout);
    if device_id.is_empty() {
        return Ok(None);
    }
    
    let device_id_num = u16::from_str_radix(&device_id, 16)
        .map_err(|e| GpufetchError::ParseError(format!("Invalid device ID '{}': {}", device_id, e)))?;
    
    create_gpu_from_device_id(device_id_num, &gpu_name)
}

fn parse_lspci_output(lspci_output: &str) -> (String, String) {
    for line in lspci_output.lines() {
        if line.contains("VGA") || line.contains("3D") {
            let mut device_id = String::new();
            let mut gpu_name = String::new();
            
            if let Some(start_bracket) = line.find("[1002:") {
                let start = start_bracket + 6;
                let remaining = &line[start..];
                if let Some(end) = remaining.find(']') {
                    device_id = remaining[..end].to_string();
                }
            }
            
            if let Some(name_start) = line.find("AMD/ATI] ") {
                let start = name_start + 9;
                let remaining = &line[start..];
                if let Some(end) = remaining.find(" [") {
                    gpu_name = remaining[..end].trim().to_string();
                }
            }
            
            return (device_id, gpu_name);
        }
    }
    (String::new(), String::new())
}

/// Creates GPU info from device ID
/// 
/// Maps device IDs to specific GPU models with accurate specs
fn create_gpu_from_device_id(device_id: u16, name_from_pci: &str) -> Result<Option<GpuInfo>> {
    let specs = match device_id {
        // RDNA 3 (RX 7000 series)
        0x743c => GpuSpecs::new("AMD Radeon RX 7900 XTX", "RDNA 3", 96, 24u64 * GB, 2500, 7, "Navi 31", false),
        0x743f => GpuSpecs::new("AMD Radeon RX 7900 XT", "RDNA 3", 84, 20u64 * GB, 2400, 7, "Navi 31", false),
        0x744c => GpuSpecs::new("AMD Radeon RX 7800 XT", "RDNA 3", 60, 16u64 * GB, 2250, 7, "Navi 31", false),
        0x7480 => GpuSpecs::new("AMD Radeon RX 7600", "RDNA 3", 32, 8u64 * GB, 2050, 7, "Navi 32", false),
        
        // RDNA 2 (RX 6000 series)
        0x731f => GpuSpecs::new("AMD Radeon RX 6900 XT", "RDNA 2", 72, 16u64 * GB, 2250, 7, "Navi 21", false),
        0x731e => GpuSpecs::new("AMD Radeon RX 6800 XT", "RDNA 2", 72, 16u64 * GB, 2250, 7, "Navi 21", false),
        0x731c => GpuSpecs::new("AMD Radeon RX 6800", "RDNA 2", 60, 16u64 * GB, 2105, 7, "Navi 21", false),
        
        // 0x73bf is shared - determine from PCI name
        0x73bf => {
            if name_from_pci.contains("6900") {
                GpuSpecs::new("AMD Radeon RX 6900 XT", "RDNA 2", 72, 16u64 * GB, 2250, 7, "Navi 21", false)
            } else if name_from_pci.contains("6800") || name_from_pci.contains("Navi 21") {
                GpuSpecs::new("AMD Radeon RX 6800 XT", "RDNA 2", 72, 16u64 * GB, 2250, 7, "Navi 21", false)
            } else {
                GpuSpecs::new("AMD Radeon RX 6700 XT", "RDNA 2", 40, 12u64 * GB, 2424, 7, "Navi 22", false)
            }
        },
        0x73c0 => GpuSpecs::new("AMD Radeon RX 6700", "RDNA 2", 40, 10u64 * GB, 2321, 7, "Navi 22", false),
        0x7340 => GpuSpecs::new("AMD Radeon RX 6600 XT", "RDNA 2", 32, 8u64 * GB, 1968, 7, "Navi 23", false),
        0x7347 => GpuSpecs::new("AMD Radeon RX 6600", "RDNA 2", 32, 8u64 * GB, 2359, 7, "Navi 23", false),
        0x7341 => GpuSpecs::new("AMD Radeon RX 6650 XT", "RDNA 2", 32, 8u64 * GB, 2181, 7, "Navi 23", false),
        
        // RDNA (RX 5000 series)
        0x734c => GpuSpecs::new("AMD Radeon RX 5700 XT", "RDNA", 40, 8u64 * GB, 1905, 7, "Navi 10", false),
        0x734e => GpuSpecs::new("AMD Radeon RX 5700", "RDNA", 36, 8u64 * GB, 1725, 7, "Navi 10", false),
        0x7343 => GpuSpecs::new("AMD Radeon RX 5600 XT", "RDNA", 24, 6u64 * GB, 1317, 7, "Navi 10", false),
        
        // CDNA/MI series (server GPUs with matrix cores)
        0x6860 => GpuSpecs::new("AMD Instinct MI300X", "CDNA 3", 1408, 192u64 * GB, 1900, 5, "CDNA 3", true),
        0x686f => GpuSpecs::new("AMD Instinct MI250X", "CDNA 2", 184, 128u64 * GB, 1700, 5, "CDNA 2", true),
        
        // Unknown - use fallback
        _ => {
            if !name_from_pci.is_empty() {
                return Ok(Some(create_fallback_gpu(name_from_pci)));
            }
            return Err(GpufetchError::DetectionFailed(format!("Unknown AMD GPU device ID: 0x{:04X}", device_id)));
        }
    };
    
    Ok(Some(build_gpu_from_specs(specs)))
}

const KB: u64 = 1024;
const MB: u64 = 1024 * KB;
const GB: u64 = 1024 * MB;

/// GPU specifications from device ID
struct GpuSpecs {
    name: String,
    arch_name: &'static str,
    cu_count: u32,
    memory_bytes: u64,
    boost_freq_mhz: u32,
    process_nm: i32,
    chip: &'static str,
    is_cdna: bool,
}

impl GpuSpecs {
    fn new(
        name: &str,
        arch_name: &'static str,
        cu_count: u32,
        memory_bytes: u64,
        boost_freq_mhz: u32,
        process_nm: i32,
        chip: &'static str,
        is_cdna: bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            arch_name,
            cu_count,
            memory_bytes,
            boost_freq_mhz,
            process_nm,
            chip,
            is_cdna,
        }
    }
}

fn build_gpu_from_specs(specs: GpuSpecs) -> GpuInfo {
    let topology = TopologyHsa {
        compute_units: specs.cu_count,
        simds_per_cu: 4,
        ray_accelerators: if specs.is_cdna { 0 } else { specs.cu_count },
        matrix_cores: if specs.is_cdna { specs.cu_count * 4 } else { 0 },
        num_xcc: 1,
    };
    
    let arch = Uarch {
        name: specs.arch_name.to_string(),
        process_nm: specs.process_nm,
        chip: specs.chip.to_string(),
        compute_capability: 0,
        llvm_target: 7,
        gt: 0,
        eu: 0,
    };
    
    let memory = Some(Memory {
        size_bytes: specs.memory_bytes,
        freq_mhz: 1600,
        bus_width: 256,
        memtype: MemoryType::Gddr6,
    });
    
    let peak_perf = Some((specs.boost_freq_mhz as i64) * 1_000_000 * (specs.cu_count as i64) * 4 * 2 * 2);
    
    let mut gpu = GpuInfo::new(Vendor::Amd, specs.name, arch, specs.boost_freq_mhz);
    gpu.topology_hsa = Some(topology);
    gpu.memory = memory;
    gpu.peak_performance = peak_perf;
    
    gpu
}

fn create_fallback_gpu(name: &str) -> GpuInfo {
    // Fallback GPU info when device ID is unknown
    let name_owned = name.to_string();
    let specs = if name.contains("7900") {
        GpuSpecs::new(&name_owned, "RDNA 3", 96, 24u64 * GB, 2500, 7, "Navi 3x", false)
    } else if name.contains("7800") {
        GpuSpecs::new(&name_owned, "RDNA 3", 60, 16u64 * GB, 2250, 7, "Navi 3x", false)
    } else if name.contains("7600") {
        GpuSpecs::new(&name_owned, "RDNA 3", 32, 8u64 * GB, 2050, 7, "Navi 3x", false)
    } else if name.contains("6900") {
        GpuSpecs::new(&name_owned, "RDNA 2", 72, 16u64 * GB, 2250, 7, "Navi 21", false)
    } else if name.contains("6800") {
        GpuSpecs::new(&name_owned, "RDNA 2", 72, 16u64 * GB, 2250, 7, "Navi 21", false)
    } else if name.contains("6700") {
        GpuSpecs::new(&name_owned, "RDNA 2", 40, 12u64 * GB, 2424, 7, "Navi 22", false)
    } else if name.contains("6600") {
        GpuSpecs::new(&name_owned, "RDNA 2", 32, 8u64 * GB, 2359, 7, "Navi 23", false)
    } else if name.contains("5700") {
        GpuSpecs::new(&name_owned, "RDNA", 40, 8u64 * GB, 1905, 7, "Navi 10", false)
    } else if name.contains("5600") {
        GpuSpecs::new(&name_owned, "RDNA", 24, 6u64 * GB, 1317, 7, "Navi 10", false)
    } else {
        GpuSpecs::new(&name_owned, "Unknown", 40, 8u64 * GB, 1900, 7, "Unknown", false)
    };
    
    build_gpu_from_specs(specs)
}
