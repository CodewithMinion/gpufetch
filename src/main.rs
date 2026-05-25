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

// Re-export commonly used items
pub use global::{error, warn, info, debug};
pub use error::{Result, GpufetchError};

use clap::Parser;
use std::process;

use args::Args;
use printer::print_gpu_info;

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
    
    if let Err(e) = fetch_and_print_gpu(gpu_idx) {
        error(&format!("Failed to fetch GPU info: {e}"));
        process::exit(1);
    }
}

fn list_gpus() -> Result<()> {
    #[cfg(feature = "intel")]
    intel::list_gpus()?;
    
    #[cfg(feature = "amd")]
    amd::list_gpus()?;
    
    #[cfg(feature = "cuda")]
    cuda::list_gpus()?;
    
    Ok(())
}

fn fetch_and_print_gpu(idx: i32) -> Result<()> {
    let mut found = false;
    
    // Try Intel backend
    #[cfg(feature = "intel")]
    {
        if idx == -1 || idx == 0 {
            match intel::get_gpu_info(0) {
                Ok(Some(gpu)) => {
                    print_gpu_info(&gpu)?;
                    found = true;
                },
                Ok(None) | Err(_) if idx != -1 => {},
                Ok(None) | Err(_) => {},
            }
        }
    }
    
    // Try AMD backend
    #[cfg(feature = "amd")]
    {
        if idx == -1 || idx == 0 {
            match amd::get_gpu_info(0) {
                Ok(Some(gpu)) => {
                    print_gpu_info(&gpu)?;
                    found = true;
                },
                Ok(None) | Err(_) if idx != -1 => {},
                Ok(None) | Err(_) => {},
            }
        }
    }
    
    // Try CUDA backend
    #[cfg(feature = "cuda")]
    {
        let mut cuda_idx = 0;
        loop {
            if idx == -1 || cuda_idx == idx {
                match cuda::get_gpu_info(cuda_idx) {
                    Ok(Some(gpu)) => {
                        print_gpu_info(&gpu)?;
                        found = true;
                    },
                    Ok(None) if idx != -1 => break,
                    Ok(None) | Err(_) => cuda_idx += 1,
                }
            } else if idx != -1 {
                break;
            }
            cuda_idx += 1;
            if cuda_idx > 100 {
                break;
            }
        }
    }
    
    if !found {
        return Err(GpufetchError::NoGpuFound);
    }
    
    Ok(())
}

