//! Intel GPU backend
//! Supports Intel integrated GPUs (Gen6+) and discrete Arc GPUs
//!
//! # Examples
//!
//! ```no_run
//! use gpufetch::intel::get_gpu_info;
//! let gpu = get_gpu_info(0)?;
//! ```

use crate::common;
use crate::gpu::{GpuInfo, Vendor, Uarch, TopologyIntel};
use crate::global::{info, warn};
use crate::error::Result;
use std::fs;

/// Lists all Intel GPUs found via lspci
pub fn list_gpus() -> Result<()> {
    info("Listing Intel GPUs via PCI...");
    
    let Some(output) = common::try_output("lspci", &["-d", "8086:"]) else {
        warn("lspci not found (install pciutils)");
        return Ok(());
    };

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

/// Gets Intel GPU information by index
pub fn get_gpu_info(_idx: i32) -> Result<Option<GpuInfo>> {
    #[cfg(target_os = "linux")]
    {
        let intel_gpu = detect_intel_gpu()?;
        
        if let Some(gpu) = intel_gpu {
            info(&format!("Found Intel GPU: {}", gpu.name));
            return Ok(Some(gpu));
        }
    }
    
    Ok(None)
}

#[cfg(target_os = "linux")]
fn detect_intel_gpu() -> Result<Option<GpuInfo>> {
    let pci_devices = read_pci_devices()?;
    
    for device in &pci_devices {
        if device.vendor_id == 0x8086 {  // Intel
            if device.device_class == 0x0300 || device.device_class == 0x0302 {
                return create_intel_gpu_info(device);
            }
        }
    }
    
    Ok(None)
}

#[cfg(target_os = "linux")]
fn read_pci_devices() -> Result<Vec<PciDevice>> {
    let mut devices = Vec::new();
    
    let entries = match fs::read_dir("/sys/bus/pci/devices") {
        Ok(e) => e,
        Err(e) => {
            warn(&format!("Cannot read PCI devices: {}", e));
            return Ok(devices);
        }
    };
    
    for entry in entries.flatten() {
        let path = entry.path();
        let vendor = match fs::read_to_string(path.join("vendor")) {
            Ok(v) => v,
            Err(_) => continue,
        };
        
        if vendor.trim() != "0x8086" {
            continue;
        }
        
        let device = match fs::read_to_string(path.join("device")) {
            Ok(d) => d,
            Err(_) => continue,
        };
        
        let class = match fs::read_to_string(path.join("class")) {
            Ok(c) => c,
            Err(_) => continue,
        };
        
        let vendor_id = u16::from_str_radix(vendor.trim().trim_start_matches("0x"), 16)
            .unwrap_or(0);
        let device_code = u16::from_str_radix(device.trim().trim_start_matches("0x"), 16)
            .unwrap_or(0);
        let device_class = u32::from_str_radix(class.trim().trim_start_matches("0x"), 16)
            .unwrap_or(0);
        
        if vendor_id == 0x8086 && (device_code != 0 || device_class != 0) {
            devices.push(PciDevice {
                vendor_id,
                device_code,
                device_class,
            });
        }
    }
    
    Ok(devices)
}

#[cfg(target_os = "linux")]
fn create_intel_gpu_info(device: &PciDevice) -> Result<Option<GpuInfo>> {
    let (name, arch_name, gen, gt, eu) = match_device_id(device.device_code);
    
    if arch_name.is_empty() {
        warn(&format!("Unknown Intel GPU device ID: 0x{:04X}", device.device_code));
        return Ok(None);
    }
    
    let freq_mhz = get_max_frequency(&arch_name);
    
    let peak_perf = if eu > 0 && freq_mhz > 0 {
        Some((freq_mhz as i64) * 1_000_000 * (eu as i64) * 2)
    } else {
        None
    };
    
    let topology = TopologyIntel {
        subslices: if gt > 0 { (gt * 2) as i32 } else { -1 },
        eu_per_subslice: if eu > 0 && gt > 0 { eu as i32 / gt as i32 / 2 } else { -1 },
        gt: gt as i32,
    };
    
    let arch = Uarch {
        name: arch_name,
        process_nm: get_process_node(gen),
        chip: format!("Gen{}", gen),
        compute_capability: 0,
        llvm_target: 0,
        gt: gt as i32,
        eu: eu as i32,
    };
    
    let mut gpu = GpuInfo::new(Vendor::Intel, name, arch, freq_mhz);
    gpu.topology_intel = Some(topology);
    gpu.peak_performance = peak_perf;
    
    Ok(Some(gpu))
}

fn match_device_id(device_id: u16) -> (String, String, u8, u8, u16) {
    // Intel GPU device ID mapping with accurate specs
    // Format: (name, arch_name, generation, GT type, EUs)
    
    // Gen6 (Sandy Bridge) - 32nm
    if (0x0102..=0x0106).contains(&device_id) {
        return ("Intel HD Graphics 2000".to_string(), "Sandy Bridge".to_string(), 6, 1, 6);
    }
    if (0x0112..=0x0116).contains(&device_id) {
        return ("Intel HD Graphics 3000".to_string(), "Sandy Bridge".to_string(), 6, 2, 12);
    }
    
    // Gen7 (Ivy Bridge) - 22nm
    if (0x0152..=0x0156).contains(&device_id) {
        return ("Intel HD Graphics 2500".to_string(), "Ivy Bridge".to_string(), 7, 1, 6);
    }
    if (0x0162..=0x0166).contains(&device_id) {
        return ("Intel HD Graphics 4000".to_string(), "Ivy Bridge".to_string(), 7, 2, 16);
    }
    
    // Gen7.5 (Haswell) - 22nm
    if (0x0402..=0x0406).contains(&device_id) {
        return ("Intel HD Graphics 4200".to_string(), "Haswell".to_string(), 7, 1, 10);
    }
    if (0x0412..=0x0416).contains(&device_id) {
        return ("Intel HD Graphics 4400".to_string(), "Haswell".to_string(), 7, 2, 20);
    }
    if (0x0422..=0x0426).contains(&device_id) {
        return ("Intel HD Graphics 4600".to_string(), "Haswell".to_string(), 7, 3, 40);
    }
    
    // Gen8 (Broadwell) - 14nm
    if (0x1602..=0x1606).contains(&device_id) {
        return ("Intel HD Graphics 5300".to_string(), "Broadwell".to_string(), 8, 1, 12);
    }
    if (0x1612..=0x1616).contains(&device_id) {
        return ("Intel HD Graphics 5500".to_string(), "Broadwell".to_string(), 8, 2, 24);
    }
    
    // Gen9 (Skylake/Kaby Lake) - 14nm
    if (0x5912..=0x5916).contains(&device_id) {
        return ("Intel HD Graphics 530".to_string(), "Skylake".to_string(), 9, 2, 24);
    }
    if 0x591b == device_id {
        return ("Intel HD Graphics 530".to_string(), "Skylake".to_string(), 9, 2, 24);
    }
    if (0x5926..=0x5927).contains(&device_id) {
        return ("Intel HD Graphics 630".to_string(), "Kaby Lake".to_string(), 9, 2, 24);
    }
    
    // Gen9.5 (Coffee Lake) - 14nm
    if 0x3ea5 == device_id {
        return ("Intel UHD Graphics 630".to_string(), "Coffee Lake".to_string(), 9, 2, 24);
    }
    
    // Gen11 (Ice Lake) - 10nm
    if (0x8a10..=0x8a14).contains(&device_id) {
        return ("Intel Iris Plus Graphics".to_string(), "Ice Lake".to_string(), 11, 2, 48);
    }
    
    // Gen12 (Tiger Lake) - 10nm
    if 0x9a49 == device_id {
        return ("Intel Iris Xe Graphics".to_string(), "Tiger Lake".to_string(), 12, 2, 80);
    }
    if (0x9a60..=0x9a70).contains(&device_id) {
        return ("Intel Iris Xe Graphics".to_string(), "Tiger Lake".to_string(), 12, 2, 96);
    }
    
    // Xe HPG (DG2) - Intel 7 (10nm Enhanced)
    if (0x56a0..=0x56a1).contains(&device_id) {
        return ("Intel Arc A770".to_string(), "Xe HPG".to_string(), 12, 4, 512);
    }
    if 0x56a5 == device_id {
        return ("Intel Arc A750".to_string(), "Xe HPG".to_string(), 12, 4, 256);
    }
    if (0x56a6..=0x56a8).contains(&device_id) {
        return ("Intel Arc A580".to_string(), "Xe HPG".to_string(), 12, 4, 192);
    }
    if 0x56a2 == device_id {
        return ("Intel Arc A380".to_string(), "Xe HPG".to_string(), 12, 4, 64);
    }
    
    // Unknown
    ("Unknown Intel GPU".to_string(), String::new(), 0, 0, 0)
}

fn get_process_node(gen: u8) -> i32 {
    match gen {
        6..=7 => 22,  // 22nm
        8 => 14,      // 14nm
        9 => 14,      // 14nm
        10 => 10,     // 10nm
        11 => 10,     // 10nm
        12..=13 => 7, // 7nm (Intel 7)
        _ => -1,
    }
}

fn get_max_frequency(arch_name: &str) -> u32 {
    // Try to read from sysfs first
    let freq_paths = [
        "/sys/class/drm/card0/device/gt_max_freq_mhz",
        "/sys/class/drm/card0/device/rp0_freq_mhz",
    ];
    
    for path in &freq_paths {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(freq) = content.trim().parse::<u32>() {
                return freq;
            }
        }
    }
    
    // Model-specific boost frequencies
    match arch_name {
        "Xe HPG" => 2100,    // Arc A770/A750 boost
        "Tiger Lake" => 1450, // Iris Xe boost
        "Ice Lake" => 1250,   // Iris Plus boost
        "Coffee Lake" => 1200, // UHD 630 boost
        "Kaby Lake" => 1150,  // HD 630 boost
        "Skylake" => 1150,    // HD 530 boost
        "Broadwell" => 1100,  // HD 5500 boost
        "Haswell" => 1200,    // HD 4600 boost
        "Ivy Bridge" => 1150, // HD 4000 boost
        "Sandy Bridge" => 1100, // HD 3000 boost
        _ => 1150,  // Default
    }
}

#[cfg(target_os = "linux")]
struct PciDevice {
    vendor_id: u16,
    device_code: u16,
    device_class: u32,
}
