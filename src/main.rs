mod common;
mod apple;
mod gpu;
mod intel;
mod amd;
mod cuda;
mod global;
mod ascii;
mod printer;
mod args;
mod colors;
mod error;

#[cfg(target_os = "macos")]
mod macos;

// Re-export commonly used items
pub use global::{error, warn, info, debug};
pub use error::{Result, GpufetchError};

use clap::Parser;
use std::process;

use args::Args;
use printer::{print_gpu_info, PrintOptions};

fn main() {
    let args = Args::parse();
    
    // Initialize global state
    global::init(args.verbose);
    
    // Handle help/version
    if args.version {
        println!("gpufetch v{}", env!("CARGO_PKG_VERSION"));
        return;
    }
    
    if args.help {
        args::print_help();
        return;
    }
    
    // Handle list GPUs
    if args.list_gpus {
        if let Err(e) = list_gpus() {
            error(&format!("Failed to list GPUs: {}", e));
            process::exit(1);
        }
        return;
    }
    
    // Fetch and display GPU info
    let gpu_idx = if args.gpu == "all" {
        -1
    } else {
        args.gpu.parse::<i32>().unwrap_or(0)
    };
    
    let print_options = PrintOptions {
        color: args.color.clone(),
        logo_short: args.logo_short,
        logo_long: args.logo_long,
    };
    
    if let Err(e) = fetch_and_print_gpu(gpu_idx, &print_options) {
        error(&format!("Failed to fetch GPU info: {e}"));
        process::exit(1);
    }
}

fn list_gpus() -> Result<()> {
    #[cfg(target_os = "macos")]
    macos::list_gpus()?;
    
    #[cfg(target_os = "linux")]
    apple::linux::list_gpus()?;

    #[cfg(feature = "intel")]
    {
        #[cfg(target_os = "linux")]
        intel::list_gpus()?;
    }
    
    #[cfg(feature = "amd")]
    {
        #[cfg(target_os = "linux")]
        amd::list_gpus()?;
    }
    
    #[cfg(not(target_os = "macos"))]
    cuda::list_gpus()?;
    
    Ok(())
}

fn fetch_and_print_gpu(idx: i32, options: &PrintOptions) -> Result<()> {
    let mut found = false;
    
    // macOS: system_profiler backend
    #[cfg(target_os = "macos")]
    {
        if idx == -1 {
            let mut mac_idx = 0;
            loop {
                match macos::get_gpu_info(mac_idx) {
                    Ok(Some(gpu)) => {
                        print_gpu_info(&gpu, options)?;
                        found = true;
                        mac_idx += 1;
                    }
                    Ok(None) => break,
                    Err(e) => {
                        debug(&format!("macOS detection failed: {e}"));
                        break;
                    }
                }
            }
        } else {
            match macos::get_gpu_info(idx) {
                Ok(Some(gpu)) => {
                    print_gpu_info(&gpu, options)?;
                    found = true;
                }
                Ok(None) => {}
                Err(e) => debug(&format!("macOS detection failed: {e}")),
            }
        }
    }
    
    // Linux on Apple hardware (Asahi, etc.): device-tree / DRM, not system_profiler
    #[cfg(target_os = "linux")]
    {
        if !found && (idx == -1 || idx == 0) {
            match apple::linux::get_gpu_info(0) {
                Ok(Some(gpu)) => {
                    print_gpu_info(&gpu, options)?;
                    found = true;
                }
                Ok(None) | Err(_) => {}
            }
        }
    }

    // Try Intel backend (Linux sysfs)
    #[cfg(all(feature = "intel", not(target_os = "macos")))]
    {
        if !found && (idx == -1 || idx == 0) {
            match intel::get_gpu_info(0) {
                Ok(Some(gpu)) => {
                    print_gpu_info(&gpu, options)?;
                    found = true;
                }
                Ok(None) | Err(_) => {}
            }
        }
    }
    
    // Try AMD backend (Linux lspci)
    #[cfg(all(feature = "amd", not(target_os = "macos")))]
    {
        if !found && (idx == -1 || idx == 0) {
            match amd::get_gpu_info(0) {
                Ok(Some(gpu)) => {
                    print_gpu_info(&gpu, options)?;
                    found = true;
                }
                Ok(None) | Err(_) => {}
            }
        }
    }
    
    // NVIDIA via nvidia-smi (Linux/Windows with proprietary drivers; not on macOS)
    #[cfg(not(target_os = "macos"))]
    if !found {
        let mut cuda_idx = 0;
        loop {
            match cuda::get_gpu_info(cuda_idx) {
                Ok(Some(gpu)) => {
                    if idx == -1 || cuda_idx == idx {
                        print_gpu_info(&gpu, options)?;
                        found = true;
                    }
                    cuda_idx += 1;
                }
                Ok(None) => break,
                Err(e) => {
                    debug(&format!("NVIDIA detection skipped: {e}"));
                    break;
                }
            }
            if cuda_idx > 100 {
                break;
            }
            if idx != -1 {
                break;
            }
        }
    }
    
    if !found {
        return Err(GpufetchError::NoGpuFound);
    }
    
    Ok(())
}
