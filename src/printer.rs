use crate::ascii::{self, Logo};
use crate::colors::{Color, RESET};
use crate::gpu::{GpuInfo, Vendor};
use crate::args::Style;
use crate::error::Result;

const MAX_TERM_WIDTH: u32 = 1024;

pub fn print_gpu_info(gpu: &GpuInfo) -> Result<()> {
    let style = Style::Fancy;
    let term_width = get_terminal_width();
    
    // Choose logo
    let logo = match gpu.vendor {
        Vendor::Nvidia => ascii::get_logo_nvidia(false),
        Vendor::Amd => ascii::get_logo_amd(false),
        Vendor::Intel => ascii::get_logo_intel(false),
    };
    
    // Prepare colors
    let logo_color = gpu.get_logo_color();
    let text_color = gpu.get_text_color();
    
    // Build attributes (including Name)
    let mut attributes = build_attributes(gpu);
    
    let max_label_len = attributes.iter()
        .map(|a| a.0.len())
        .max()
        .unwrap_or(0);
    
    // Print logo with attributes
    print_logo_with_attrs(&logo, &attributes, max_label_len, term_width, &logo_color, &text_color);
    
    Ok(())
}

fn build_attributes(gpu: &GpuInfo) -> Vec<(String, String)> {
    let mut attrs = Vec::new();
    const LABEL_WIDTH: usize = 18; // Fixed width for alignment
    
    // Name
    attrs.push((format!("{:<LABEL_WIDTH$}", "Name:"), gpu.name.clone()));
    
    // Chip/Processor
    if !gpu.arch.chip.is_empty() {
        attrs.push((format!("{:<LABEL_WIDTH$}", "GPU processor:"), gpu.arch.chip.clone()));
    }
    
    // Microarchitecture
    let uarch_str = if gpu.arch.compute_capability > 0 {
        format!("{} (CC {}.{})", gpu.arch.name, gpu.arch.compute_capability / 10, gpu.arch.compute_capability % 10)
    } else {
        gpu.arch.name.clone()
    };
    attrs.push((format!("{:<LABEL_WIDTH$}", "Microarchitecture:"), uarch_str));
    
    // Technology
    if gpu.arch.process_nm > 0 {
        attrs.push((format!("{:<LABEL_WIDTH$}", "Technology:"), format!("{} nm", gpu.arch.process_nm)));
    }
    
    // Max Frequency
    attrs.push((format!("{:<LABEL_WIDTH$}", "Max Frequency:"), format!("{} MHz", gpu.freq_mhz)));
    
    // Vendor-specific attributes
    match gpu.vendor {
        Vendor::Nvidia => {
            if let Some(topo) = &gpu.topology_cuda {
                attrs.push((format!("{:<LABEL_WIDTH$}", "SMs:"), topo.streaming_multiprocessors.to_string()));
                attrs.push((format!("{:<LABEL_WIDTH$}", "Cores/SM:"), topo.cores_per_sm.to_string()));
                attrs.push((format!("{:<LABEL_WIDTH$}", "CUDA Cores:"), topo.cuda_cores.to_string()));
                if topo.tensor_cores > 0 {
                    attrs.push((format!("{:<LABEL_WIDTH$}", "Tensor Cores:"), topo.tensor_cores.to_string()));
                }
            }
            if let Some(mem) = &gpu.memory {
                let mem_type_str = match mem.memtype {
                    crate::gpu::MemoryType::Gddr5 => "GDDR5",
                    crate::gpu::MemoryType::Gddr5x => "GDDR5X",
                    crate::gpu::MemoryType::Gddr6 => "GDDR6",
                    crate::gpu::MemoryType::Gddr6x => "GDDR6X",
                    crate::gpu::MemoryType::Hbm => "HBM",
                    crate::gpu::MemoryType::Hbm2 => "HBM2",
                    crate::gpu::MemoryType::Hbm2e => "HBM2e",
                    crate::gpu::MemoryType::Hbm3 => "HBM3",
                    crate::gpu::MemoryType::Ddr4 => "DDR4",
                    crate::gpu::MemoryType::Ddr5 => "DDR5",
                    crate::gpu::MemoryType::Unknown => "Unknown",
                };
                let size_gb = mem.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                attrs.push((format!("{:<LABEL_WIDTH$}", "Memory:"), format!("{:.1} GB {}", size_gb, mem_type_str)));
                attrs.push((format!("{:<LABEL_WIDTH$}", "Memory freq.:"), format!("{} MHz", mem.freq_mhz)));
                attrs.push((format!("{:<LABEL_WIDTH$}", "Bus width:"), format!("{} bit", mem.bus_width)));
            }
            if let Some(cache) = &gpu.cache {
                if cache.l2_size > 0 {
                    attrs.push((format!("{:<LABEL_WIDTH$}", "L2 Size:"), format!("{} KB", cache.l2_size / 1024)));
                }
            }
            if let Some(peak) = gpu.peak_performance {
                attrs.push((format!("{:<LABEL_WIDTH$}", "Peak Perf.:"), format!("{} TFLOPS", peak as f64 / 1_000_000_000_000.0)));
            }
        },
        Vendor::Amd => {
            if let Some(topo) = &gpu.topology_hsa {
                attrs.push((format!("{:<LABEL_WIDTH$}", "CUs:"), topo.compute_units.to_string()));
                // RDNA has Ray Accelerators, CDNA has Matrix Cores
                if topo.ray_accelerators > 0 {
                    attrs.push((format!("{:<LABEL_WIDTH$}", "Ray Accelerators:"), topo.ray_accelerators.to_string()));
                } else if topo.matrix_cores > 0 {
                    attrs.push((format!("{:<LABEL_WIDTH$}", "Matrix Cores:"), topo.matrix_cores.to_string()));
                }
                if topo.num_xcc > 1 {
                    attrs.push((format!("{:<LABEL_WIDTH$}", "XCDs:"), topo.num_xcc.to_string()));
                }
            }
            if let Some(mem) = &gpu.memory {
                let size_gb = mem.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                attrs.push((format!("{:<LABEL_WIDTH$}", "Memory:"), format!("{:.1} GB", size_gb)));
                attrs.push((format!("{:<LABEL_WIDTH$}", "Bus width:"), format!("{} bit", mem.bus_width)));
            }
        },
        Vendor::Intel => {
            if let Some(topo) = &gpu.topology_intel {
                if topo.gt > 0 {
                    attrs.push((format!("{:<LABEL_WIDTH$}", "GT:"), format!("GT{}", topo.gt)));
                }
                if topo.subslices > 0 && topo.eu_per_subslice > 0 {
                    let eus = topo.subslices * topo.eu_per_subslice;
                    attrs.push((format!("{:<LABEL_WIDTH$}", "EUs:"), eus.to_string()));
                }
            }
            if let Some(peak) = gpu.peak_performance {
                attrs.push((format!("{:<LABEL_WIDTH$}", "Peak Perf.:"), format!("{} GFLOPS", peak as f64 / 1_000_000_000.0)));
            }
        },
    }
    
    attrs
}

fn print_logo_with_attrs(
    logo: &Logo,
    attributes: &[(String, String)],
    max_label_len: usize,
    _term_width: u32,
    logo_color: &Color,
    text_color: &Color,
) {
    let mut lines: Vec<&str> = logo.art.lines().collect();
    
    // Pad logo lines if needed
    while lines.len() < attributes.len() {
        lines.push("");
    }
    
    let logo_lines = lines.len();
    
    // Align attributes to the right side of logo, offset by 2 lines
    let attr_start: usize = 2;
    
    let mut line_idx: usize = 0;
    
    for (i, line) in logo.art.lines().enumerate() {
        // Parse the line and replace $C1/$C2 with actual colors
        let mut output = String::new();
        let mut chars = line.chars().peekable();
        let mut current_color = logo_color;
        let mut first_char = true;
        
        while let Some(c) = chars.next() {
            if c == '$' {
                if let Some(&next) = chars.peek() {
                    chars.next();
                    match next {
                        'C' => {
                            if let Some(&color_num) = chars.peek() {
                                chars.next();
                                match color_num {
                                    '1' => current_color = logo_color,
                                    '2' => current_color = text_color,
                                    _ => {}
                                }
                            }
                        },
                        _ => {
                            output.push('$');
                            output.push(next);
                        }
                    }
                }
            } else if c != ' ' && c != '\t' {
                // Non-whitespace character - apply color at start of line
                if first_char {
                    output.push_str(&current_color.to_ansi_fg(false));
                    first_char = false;
                }
                output.push(c);
            } else {
                // Whitespace - reset color if we had one
                if !first_char {
                    output.push_str(RESET);
                    output.push(c);
                    output.push_str(&current_color.to_ansi_fg(false));
                    first_char = true;
                } else {
                    output.push(c);
                }
            }
        }
        
        // Print the logo line
        print!("{}", output);
        
        // Print attribute if this is in the attribute range
        if i >= attr_start && (i - attr_start) < attributes.len() {
            let attr_idx = i - attr_start;
            let (label, value) = &attributes[attr_idx];
            print!("          {}{}{}{}{}", logo_color.to_ansi_fg(false), label, RESET, text_color.to_ansi_fg(false), value);
        }
        
        println!("{}", RESET);
        line_idx += 1;
    }
    
    // No remaining attributes - all printed in loop
    
    println!();
}

fn get_terminal_width() -> u32 {
    use std::mem::MaybeUninit;
    use libc::{ioctl, STDOUT_FILENO, winsize, TIOCGWINSZ};
    
    unsafe {
        let mut ws: winsize = MaybeUninit::uninit().assume_init();
        if ioctl(STDOUT_FILENO, TIOCGWINSZ.into(), &mut ws) == 0 {
            return ws.ws_col as u32;
        }
    }
    MAX_TERM_WIDTH
}
