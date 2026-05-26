//! GPU information structures and types
//!
//! This module defines the core data structures for representing GPU information
//! across all vendors (NVIDIA, AMD, Intel).

use crate::colors::Color;

/// GPU vendor identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vendor {
    Nvidia,
    Amd,
    Intel,
    #[allow(dead_code)]
    Apple,
}

/// GPU memory information
#[derive(Debug, Clone)]
pub struct Memory {
    pub size_bytes: u64,
    pub freq_mhz: u32,
    pub bus_width: u32,
    pub memtype: MemoryType,
}

/// GPU memory type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemoryType {
    #[allow(dead_code)]
    Unknown,
    Gddr5,
    Gddr5x,
    Gddr6,
    Gddr6x,
    #[allow(dead_code)]
    Hbm,
    #[allow(dead_code)]
    Hbm2,
    #[allow(dead_code)]
    Hbm2e,
    #[allow(dead_code)]
    Hbm3,
    #[allow(dead_code)]
    Ddr4,
    #[allow(dead_code)]
    Ddr5,
}

/// GPU cache information
#[derive(Debug, Clone)]
pub struct CacheInfo {
    pub l1_size: u32,
    pub l2_size: u64,
    #[allow(dead_code)]
    pub shared_mem_per_block: u32,
}

/// CUDA-specific topology (NVIDIA)
#[derive(Debug, Clone)]
pub struct TopologyCuda {
    pub streaming_multiprocessors: u32,
    pub cores_per_sm: u32,
    pub cuda_cores: u32,
    pub tensor_cores: u32,
}

/// HSA topology (AMD RDNA/CDNA)
///
/// RDNA architectures have Ray Accelerators, while CDNA has Matrix Cores
#[derive(Debug, Clone)]
pub struct TopologyHsa {
    pub compute_units: u32,
    pub simds_per_cu: u32,
    /// Ray Accelerators (RDNA 1/2/3) - 1 per CU
    pub ray_accelerators: u32,
    /// Matrix Cores (CDNA) - 4 per CU for AI compute
    pub matrix_cores: u32,
    pub num_xcc: u32,
}

/// Intel GPU topology
#[derive(Debug, Clone)]
pub struct TopologyIntel {
    pub subslices: i32,
    pub eu_per_subslice: i32,
    pub gt: i32,
}

/// Apple Silicon GPU topology
#[derive(Debug, Clone)]
pub struct TopologyApple {
    pub gpu_cores: u32,
}

/// Microarchitecture information
#[derive(Debug, Clone)]
pub struct Uarch {
    pub name: String,
    pub process_nm: i32,
    pub chip: String,
    /// CUDA compute capability (NVIDIA only)
    pub compute_capability: u32,
    /// LLVM target version (AMD only)
    #[allow(dead_code)]
    pub llvm_target: i32,
    /// GT type (Intel only)
    #[allow(dead_code)]
    pub gt: i32,
    /// Execution units (Intel only)
    #[allow(dead_code)]
    pub eu: i32,
}

/// Complete GPU information
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub vendor: Vendor,
    pub name: String,
    pub arch: Uarch,
    pub freq_mhz: u32,
    pub memory: Option<Memory>,
    pub cache: Option<CacheInfo>,
    pub topology_cuda: Option<TopologyCuda>,
    pub topology_hsa: Option<TopologyHsa>,
    pub topology_intel: Option<TopologyIntel>,
    pub topology_apple: Option<TopologyApple>,
    /// Peak FP32 performance in FLOPS
    pub peak_performance: Option<i64>,
    /// Peak tensor/Matrix performance in FLOPS
    pub peak_performance_tensor: Option<i64>,
}

impl GpuInfo {
    /// Creates a new GpuInfo instance with vendor-specific defaults
    pub fn new(vendor: Vendor, name: String, arch: Uarch, freq_mhz: u32) -> Self {
        Self {
            vendor,
            name,
            arch,
            freq_mhz,
            memory: None,
            cache: None,
            topology_cuda: None,
            topology_hsa: None,
            topology_intel: None,
            topology_apple: None,
            peak_performance: None,
            peak_performance_tensor: None,
        }
    }
    
    /// Returns the vendor name as a static string
    pub fn get_vendor_name(&self) -> &'static str {
        match self.vendor {
            Vendor::Nvidia => "NVIDIA",
            Vendor::Amd => "AMD",
            Vendor::Intel => "Intel",
            Vendor::Apple => "Apple",
        }
    }
    
    /// Returns the appropriate logo color for this vendor
    pub fn get_logo_color(&self) -> Color {
        match self.vendor {
            Vendor::Nvidia => Color::nvidia_logo(),
            Vendor::Amd => Color::amd_logo(),
            Vendor::Intel => Color::intel_logo(),
            Vendor::Apple => Color::apple_logo(),
        }
    }
    
    /// Returns the appropriate text color for this vendor
    pub fn get_text_color(&self) -> Color {
        match self.vendor {
            Vendor::Nvidia => Color::nvidia_text(),
            Vendor::Amd => Color::amd_text(),
            Vendor::Intel => Color::intel_text(),
            Vendor::Apple => Color::apple_text(),
        }
    }
}
