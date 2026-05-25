use std::env;

fn main() {
    // Check for CUDA availability
    if env::var("CARGO_FEATURE_CUDA").is_ok() {
        println!("cargo:rustc-check-cfg=cfg(feature, values(\"cuda\", \"intel\", \"amd\"))");
        
        // Check if CUDA is installed
        if std::path::Path::new("/usr/local/cuda").exists() || 
           std::env::var("CUDA_HOME").is_ok() {
            println!("cargo:warning=CUDA detected - CUDA backend can be enabled");
        } else {
            println!("cargo:warning=CUDA not found - CUDA backend will be disabled");
        }
    }
    
    // Check for PCI utilities
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=pci");
        
        // Check if pciutils is available
        match std::process::Command::new("pkg-config")
            .arg("--exists")
            .arg("libpci")
            .status() {
            Ok(status) => {
                if status.success() {
                    println!("cargo:rustc-link-lib=pci");
                }
            },
            Err(_) => {
                println!("cargo:warning=pciutils may not be available");
            }
        }
    }
    
    // Rebuild if src files change
    println!("cargo:rerun-if-changed=src/");
    println!("cargo:rerun-if-changed=Cargo.toml");
}
