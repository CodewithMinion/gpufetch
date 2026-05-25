//! NVIDIA CUDA backend
//! Requires CUDA toolkit to be installed
//!
//! # Examples
//!
//! ```no_run
//! use gpufetch::cuda::get_gpu_info;
//! let gpu = get_gpu_info(0)?;
//! ```

use crate::gpu::{GpuInfo, Vendor, Uarch, TopologyCuda, Memory, MemoryType, CacheInfo};
use crate::global::{info, warn};
use crate::error::{Result, GpufetchError};
#[cfg(target_os = "linux")]
use std::process::Command;

/// Lists all NVIDIA GPUs found via nvidia-smi
pub fn list_gpus() -> Result<()> {
    info("Listing NVIDIA GPUs via CUDA...");
    
    #[cfg(not(feature = "cuda"))]
    {
        warn("CUDA feature not enabled - using nvidia-smi fallback");
    }
    
    #[cfg(feature = "cuda")]
    {
        if !cuda_available() {
            warn("CUDA runtime not available");
            return Ok(());
        }
    }
        
    let output = Command::new("nvidia-smi")
        .arg("-L")
        .output()
        .map_err(|e| GpufetchError::CommandFailed(e))?;
    
    if !output.status.success() {
        warn("nvidia-smi returned non-zero exit code");
        return Ok(());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        println!("{line}");
    }
    
    Ok(())
}

/// Gets NVIDIA GPU information by index
pub fn get_gpu_info(idx: i32) -> Result<Option<GpuInfo>> {
    #[cfg(not(feature = "cuda"))]
    {
        detect_via_nvidia_smi(idx)
    }
    
    #[cfg(feature = "cuda")]
    {
        if !cuda_available() {
            return Ok(None);
        }
        
        match get_cuda_info_via_smi(idx) {
            Ok(Some(gpu)) => {
                info(&format!("Found NVIDIA GPU: {}", gpu.name));
                Ok(Some(gpu))
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(feature = "cuda")]
fn cuda_available() -> bool {
    std::path::Path::new("/usr/local/cuda").exists() || 
    std::env::var("CUDA_HOME").is_ok()
}

#[cfg(feature = "cuda")]
fn get_cuda_info_via_smi(idx: i32) -> Result<Option<GpuInfo>> {
    let output = Command::new("nvidia-smi")
        .args(&["--query-gpu=name,memory.total,driver_version", "--format=csv,noheader"])
        .output()
        .map_err(|e| GpufetchError::CommandFailed(e))?;
    
    if !output.status.success() {
        return Ok(None);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    
    if idx as usize >= lines.len() {
        return Ok(None);
    }
    
    let parts: Vec<&str> = lines[idx as usize].split(", ").collect();
    if parts.len() < 2 {
        return Err(GpufetchError::ParseError("Invalid nvidia-smi output".to_string()));
    }
    
    let name = parts[0].trim().to_string();
    let memory_total = parts[1].trim();
    
    let memory_bytes: u64 = memory_total
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .map(|mb| mb * 1024 * 1024)
        .unwrap_or(0);
    
    let (arch, compute_cap, cuda_cores, sm_count, freq_mhz) = get_gpu_specs(&name);
    
    if cuda_cores == 0 {
        return Ok(None);
    }
    
    let topology = TopologyCuda {
        streaming_multiprocessors: sm_count,
        cores_per_sm: get_cores_per_sm(compute_cap),
        cuda_cores,
        tensor_cores: if compute_cap >= 70 { get_tensor_cores(sm_count, compute_cap) } else { 0 },
    };
    
    let arch_struct = Uarch {
        name: arch.clone(),
        process_nm: get_process_node(&arch),
        chip: format!("CC {}.{}", compute_cap / 10, compute_cap % 10),
        compute_capability: compute_cap,
        llvm_target: 0,
        gt: 0,
        eu: 0,
    };
    
    let memory = if memory_bytes > 0 {
        Some(Memory {
            size_bytes: memory_bytes,
            freq_mhz: get_memory_frequency(&name),
            bus_width: get_memory_bus_width(&name),
            memtype: guess_memory_type(&name, &arch),
        })
    } else {
        None
    };
    
    let cache = Some(CacheInfo {
        l1_size: get_l1_size(compute_cap),
        l2_size: get_l2_size(&arch),
        shared_mem_per_block: 49152,
    });
    
    let peak_perf = Some((freq_mhz as i64) * 1_000_000 * (cuda_cores as i64) * 2);
    let peak_tensor = if topology.tensor_cores > 0 {
        Some((freq_mhz as i64) * 1_000_000 * (topology.tensor_cores as i64) * 64)
    } else {
        None
    };
    
    let mut gpu = GpuInfo::new(Vendor::Nvidia, name, arch_struct, freq_mhz);
    gpu.topology_cuda = Some(topology);
    gpu.memory = memory;
    gpu.cache = cache;
    gpu.peak_performance = peak_perf;
    gpu.peak_performance_tensor = peak_tensor;
    
    Ok(Some(gpu))
}

#[cfg(not(feature = "cuda"))]
fn detect_via_nvidia_smi(idx: i32) -> Result<Option<GpuInfo>> {
    let output = Command::new("nvidia-smi")
        .args(&["--query-gpu=name", "--format=csv,noheader"])
        .output()
        .map_err(|e| GpufetchError::CommandFailed(e))?;
    
    if !output.status.success() {
        return Ok(None);
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    
    if idx as usize >= lines.len() {
        return Ok(None);
    }
    
    let name = lines[idx as usize].trim().to_string();
    
    let (arch, compute_cap, cuda_cores, sm_count, freq_mhz) = get_gpu_specs(&name);
    
    if cuda_cores == 0 {
        return Ok(None);
    }
    
    let arch_struct = Uarch {
        name: arch.clone(),
        process_nm: get_process_node(&arch),
        chip: format!("CC {}.{}", compute_cap / 10, compute_cap % 10),
        compute_capability: compute_cap,
        llvm_target: 0,
        gt: 0,
        eu: 0,
    };
    
    let topology = TopologyCuda {
        streaming_multiprocessors: sm_count,
        cores_per_sm: get_cores_per_sm(compute_cap),
        cuda_cores,
        tensor_cores: if compute_cap >= 70 { get_tensor_cores(sm_count, compute_cap) } else { 0 },
    };
    
    let peak_perf = Some((freq_mhz as i64) * 1_000_000 * (cuda_cores as i64) * 2);
    
    let mut gpu = GpuInfo::new(Vendor::Nvidia, name, arch_struct, freq_mhz);
    gpu.topology_cuda = Some(topology);
    gpu.peak_performance = peak_perf;
    
    Ok(Some(gpu))
}

fn get_gpu_specs(name: &str) -> (String, u32, u32, u32, u32) {
    // Map GPU name to architecture and specs with accurate boost clocks
    
    // RTX 40 series (Ada Lovelace) - TSMC 4N
    if name.contains("RTX 4090") {
        return ("Ada Lovelace".to_string(), 89, 16384, 128, 2520);  // Boost: 2.52 GHz
    }
    if name.contains("RTX 4080") {
        return ("Ada Lovelace".to_string(), 89, 9728, 76, 2510);  // Boost: 2.51 GHz
    }
    if name.contains("RTX 4070") {
        return ("Ada Lovelace".to_string(), 89, 5888, 46, 2480);  // Boost: 2.48 GHz
    }
    if name.contains("RTX 4060") {
        return ("Ada Lovelace".to_string(), 89, 3072, 24, 2460);  // Boost: 2.46 GHz
    }
    
    // RTX 30 series (Ampere) - Samsung 8nm
    if name.contains("RTX 3090") {
        return ("Ampere".to_string(), 86, 10496, 82, 1695);  // Boost: 1.695 GHz
    }
    if name.contains("RTX 3080") {
        return ("Ampere".to_string(), 86, 8704, 68, 1710);  // Boost: 1.71 GHz
    }
    if name.contains("RTX 3070") {
        return ("Ampere".to_string(), 86, 5888, 46, 1725);  // Boost: 1.725 GHz
    }
    if name.contains("RTX 3060") {
        return ("Ampere".to_string(), 86, 3584, 28, 1777);  // Boost: 1.777 GHz
    }
    
    // RTX 20 series (Turing) - TSMC 12nm FFN
    if name.contains("RTX 2080 Ti") {
        return ("Turing".to_string(), 75, 4352, 68, 1635);  // Boost: 1.635 GHz
    }
    if name.contains("RTX 2080") {
        return ("Turing".to_string(), 75, 2944, 46, 1710);  // Boost: 1.71 GHz
    }
    if name.contains("RTX 2070") {
        return ("Turing".to_string(), 75, 2304, 36, 1620);  // Boost: 1.62 GHz
    }
    if name.contains("RTX 2060") {
        return ("Turing".to_string(), 75, 1920, 30, 1650);  // Boost: 1.65 GHz
    }
    
    // GTX 16 series (Turing - no RT/Tensor cores)
    if name.contains("GTX 1660 Ti") {
        return ("Turing".to_string(), 75, 1536, 24, 1770);  // Boost: 1.77 GHz
    }
    if name.contains("GTX 1660") {
        return ("Turing".to_string(), 75, 1408, 22, 1785);  // Boost: 1.785 GHz
    }
    
    // GTX 10 series (Pascal) - TSMC 16nm
    if name.contains("GTX 1080 Ti") {
        return ("Pascal".to_string(), 61, 3584, 28, 1600);  // Boost: 1.6 GHz
    }
    if name.contains("GTX 1080") {
        return ("Pascal".to_string(), 61, 2560, 20, 1733);  // Boost: 1.733 GHz
    }
    if name.contains("GTX 1070") {
        return ("Pascal".to_string(), 61, 1920, 15, 1683);  // Boost: 1.683 GHz
    }
    if name.contains("GTX 1060") {
        return ("Pascal".to_string(), 61, 1280, 10, 1708);  // Boost: 1.708 GHz
    }
    
    // GTX 900 series (Maxwell) - TSMC 28nm
    if name.contains("GTX 980 Ti") {
        return ("Maxwell".to_string(), 52, 2816, 22, 1075);  // Boost: 1.075 GHz
    }
    if name.contains("GTX 980") {
        return ("Maxwell".to_string(), 52, 2048, 16, 1126);  // Boost: 1.126 GHz
    }
    if name.contains("GTX 970") {
        return ("Maxwell".to_string(), 52, 1664, 13, 1050);  // Boost: 1.05 GHz
    }
    
    // Unknown GPU - use conservative defaults
    ("Unknown".to_string(), 0, 0, 0, 1000)
}

fn get_cores_per_sm(compute_cap: u32) -> u32 {
    match compute_cap {
        c if c >= 80 => 128,  // Ampere and later
        c if c >= 70 => 64,   // Turing, Volta
        c if c >= 60 => 128,  // Pascal
        c if c >= 50 => 128,  // Maxwell
        c if c >= 30 => 192,  // Kepler
        _ => 0,
    }
}

fn get_tensor_cores(sm_count: u32, compute_cap: u32) -> u32 {
    if compute_cap >= 80 {
        sm_count * 4  // Ampere: 4 tensor cores per SM
    } else if compute_cap >= 70 {
        sm_count * 2  // Turing/Volta: 2 tensor cores per SM
    } else {
        0
    }
}

fn get_process_node(arch: &str) -> i32 {
    match arch {
        "Ada Lovelace" => 4,   // 4nm
        "Ampere" => 8,         // 8nm
        "Turing" => 12,        // 12nm
        "Pascal" => 16,        // 16nm
        "Maxwell" => 28,       // 28nm
        "Volta" => 12,         // 12nm
        _ => -1,
    }
}

fn get_memory_frequency(name: &str) -> u32 {
    if name.contains("RTX 40") {
        21000  // GDDR6X
    } else if name.contains("RTX 30") {
        1875   // GDDR6X/GDDR6
    } else if name.contains("RTX 20") {
        1750   // GDDR6
    } else if name.contains("GTX 16") {
        2000   // GDDR6
    } else if name.contains("GTX 10") {
        2002   // GDDR5X/GDDR5
    } else {
        1752   // Default
    }
}

fn get_memory_bus_width(name: &str) -> u32 {
    if name.contains("RTX 4090") {
        384
    } else if name.contains("RTX 4080") {
        256
    } else if name.contains("RTX 3090") {
        384
    } else if name.contains("RTX 3080") {
        320
    } else if name.contains("RTX 3070") {
        256
    } else if name.contains("RTX 3060") {
        192
    } else if name.contains("RTX 2080") {
        256
    } else if name.contains("RTX 2070") {
        256
    } else if name.contains("GTX 1080") {
        256
    } else if name.contains("GTX 1070") {
        256
    } else if name.contains("GTX 1060") {
        192
    } else {
        256
    }
}

fn guess_memory_type(name: &str, arch: &str) -> MemoryType {
    if name.contains("RTX 40") || name.contains("RTX 3080") || name.contains("RTX 3090") {
        MemoryType::Gddr6x
    } else if arch == "Ampere" || arch == "Turing" || arch == "Ada Lovelace" {
        MemoryType::Gddr6
    } else if arch == "Pascal" {
        MemoryType::Gddr5x
    } else {
        MemoryType::Gddr5
    }
}

fn get_l1_size(compute_cap: u32) -> u32 {
    if compute_cap >= 70 {
        128 * 1024  // 128 KB configurable
    } else if compute_cap >= 60 {
        48 * 1024   // 48 KB configurable
    } else {
        16 * 1024   // 16 KB
    }
}

fn get_l2_size(arch: &str) -> u64 {
    match arch {
        "Ada Lovelace" => 72 * 1024,  // 72 MB
        "Ampere" => 6 * 1024,         // 6 MB
        "Turing" => 2 * 1024,         // 2 MB
        "Pascal" => 256 * 1024,       // 256 KB
        _ => 256 * 1024,
    }
}
