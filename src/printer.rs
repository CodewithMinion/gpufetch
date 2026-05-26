use crate::ascii::{self, Logo};
use crate::colors::{self, Color, RESET};
use crate::gpu::{GpuInfo, Vendor};
use crate::error::Result;

const MAX_TERM_WIDTH: u32 = 1024;

#[derive(Debug, Clone, Default)]
pub struct PrintOptions {
    pub color: Option<String>,
    pub logo_short: bool,
    pub logo_long: bool,
}

pub fn print_gpu_info(gpu: &GpuInfo, options: &PrintOptions) -> Result<()> {
    let _term_width = get_terminal_width();
    let use_long_logo = options.logo_long && !options.logo_short;

    let logo = match gpu.vendor {
        Vendor::Nvidia => ascii::get_logo_nvidia(use_long_logo),
        Vendor::Amd => ascii::get_logo_amd(use_long_logo),
        Vendor::Intel | Vendor::Apple => ascii::get_logo_intel(use_long_logo),
    };

    let (logo_color, text_color) = resolve_colors(gpu, options.color.as_deref());

    let attributes = build_attributes(gpu);
    let max_label_len = attributes
        .iter()
        .map(|a| a.0.len())
        .max()
        .unwrap_or(0);

    print_logo_with_attrs(&logo, &attributes, max_label_len, &logo_color, &text_color);

    Ok(())
}

fn resolve_colors(gpu: &GpuInfo, color_arg: Option<&str>) -> (Color, Color) {
    let Some(scheme) = color_arg else {
        return (gpu.get_logo_color(), gpu.get_text_color());
    };

    if scheme.contains(':') {
        if let Ok(custom) = colors::parse_color_string(scheme) {
            let logo = custom.first().copied().unwrap_or_else(|| gpu.get_logo_color());
            let text = custom.get(3).copied().unwrap_or_else(|| gpu.get_text_color());
            return (logo, text);
        }
    }

    match colors::parse_color_scheme(scheme) {
        Ok((logo_colors, text_colors)) => (
            logo_colors.first().copied().unwrap_or_else(|| gpu.get_logo_color()),
            text_colors.first().copied().unwrap_or_else(|| gpu.get_text_color()),
        ),
        Err(_) => (gpu.get_logo_color(), gpu.get_text_color()),
    }
}

fn build_attributes(gpu: &GpuInfo) -> Vec<(String, String)> {
    let mut attrs = Vec::new();
    const LABEL_WIDTH: usize = 18;

    attrs.push((
        format!("{:<LABEL_WIDTH$}", "Name:"),
        gpu.name.clone(),
    ));
    attrs.push((
        format!("{:<LABEL_WIDTH$}", "Vendor:"),
        gpu.get_vendor_name().to_string(),
    ));

    if !gpu.arch.chip.is_empty() {
        attrs.push((
            format!("{:<LABEL_WIDTH$}", "GPU processor:"),
            gpu.arch.chip.clone(),
        ));
    }

    let uarch_str = if gpu.arch.compute_capability > 0 {
        format!(
            "{} (CC {}.{})",
            gpu.arch.name,
            gpu.arch.compute_capability / 10,
            gpu.arch.compute_capability % 10
        )
    } else {
        gpu.arch.name.clone()
    };
    attrs.push((
        format!("{:<LABEL_WIDTH$}", "Microarchitecture:"),
        uarch_str,
    ));

    if gpu.arch.process_nm > 0 {
        attrs.push((
            format!("{:<LABEL_WIDTH$}", "Technology:"),
            format!("{} nm", gpu.arch.process_nm),
        ));
    }

    attrs.push((
        format!("{:<LABEL_WIDTH$}", "Max Frequency:"),
        format!("{} MHz", gpu.freq_mhz),
    ));

    match gpu.vendor {
        Vendor::Nvidia => {
            if let Some(topo) = &gpu.topology_cuda {
                attrs.push((
                    format!("{:<LABEL_WIDTH$}", "SMs:"),
                    topo.streaming_multiprocessors.to_string(),
                ));
                attrs.push((
                    format!("{:<LABEL_WIDTH$}", "Cores/SM:"),
                    topo.cores_per_sm.to_string(),
                ));
                attrs.push((
                    format!("{:<LABEL_WIDTH$}", "CUDA Cores:"),
                    topo.cuda_cores.to_string(),
                ));
                if topo.tensor_cores > 0 {
                    attrs.push((
                        format!("{:<LABEL_WIDTH$}", "Tensor Cores:"),
                        topo.tensor_cores.to_string(),
                    ));
                }
            }
            if let Some(mem) = &gpu.memory {
                let mem_type_str = memory_type_label(mem.memtype);
                let size_gb = mem.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                attrs.push((
                    format!("{:<LABEL_WIDTH$}", "Memory:"),
                    format!("{size_gb:.1} GB {mem_type_str}"),
                ));
                if mem.freq_mhz > 0 {
                    attrs.push((
                        format!("{:<LABEL_WIDTH$}", "Memory freq.:"),
                        format!("{} MHz", mem.freq_mhz),
                    ));
                }
                if mem.bus_width > 0 {
                    attrs.push((
                        format!("{:<LABEL_WIDTH$}", "Bus width:"),
                        format!("{} bit", mem.bus_width),
                    ));
                }
            }
            if let Some(cache) = &gpu.cache {
                if cache.l1_size > 0 {
                    attrs.push((
                        format!("{:<LABEL_WIDTH$}", "L1 Size:"),
                        format!("{} KB", cache.l1_size / 1024),
                    ));
                }
                if cache.l2_size > 0 {
                    attrs.push((
                        format!("{:<LABEL_WIDTH$}", "L2 Size:"),
                        format!("{} KB", cache.l2_size / 1024),
                    ));
                }
            }
            if let Some(peak) = gpu.peak_performance {
                attrs.push((
                    format!("{:<LABEL_WIDTH$}", "Peak Perf.:"),
                    format!("{:.2} TFLOPS", peak as f64 / 1_000_000_000_000.0),
                ));
            }
            if let Some(peak) = gpu.peak_performance_tensor {
                attrs.push((
                    format!("{:<LABEL_WIDTH$}", "Tensor Perf.:"),
                    format!("{:.2} TFLOPS", peak as f64 / 1_000_000_000_000.0),
                ));
            }
        }
        Vendor::Amd => {
            if let Some(topo) = &gpu.topology_hsa {
                attrs.push((
                    format!("{:<LABEL_WIDTH$}", "CUs:"),
                    topo.compute_units.to_string(),
                ));
                attrs.push((
                    format!("{:<LABEL_WIDTH$}", "SIMDs/CU:"),
                    topo.simds_per_cu.to_string(),
                ));
                if topo.ray_accelerators > 0 {
                    attrs.push((
                        format!("{:<LABEL_WIDTH$}", "Ray Accelerators:"),
                        topo.ray_accelerators.to_string(),
                    ));
                } else if topo.matrix_cores > 0 {
                    attrs.push((
                        format!("{:<LABEL_WIDTH$}", "Matrix Cores:"),
                        topo.matrix_cores.to_string(),
                    ));
                }
                if topo.num_xcc > 1 {
                    attrs.push((
                        format!("{:<LABEL_WIDTH$}", "XCDs:"),
                        topo.num_xcc.to_string(),
                    ));
                }
            }
            if let Some(mem) = &gpu.memory {
                let size_gb = mem.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                attrs.push((
                    format!("{:<LABEL_WIDTH$}", "Memory:"),
                    format!("{size_gb:.1} GB"),
                ));
                if mem.bus_width > 0 {
                    attrs.push((
                        format!("{:<LABEL_WIDTH$}", "Bus width:"),
                        format!("{} bit", mem.bus_width),
                    ));
                }
            }
        }
        Vendor::Intel => {
            if let Some(topo) = &gpu.topology_intel {
                if topo.gt > 0 {
                    attrs.push((
                        format!("{:<LABEL_WIDTH$}", "GT:"),
                        format!("GT{}", topo.gt),
                    ));
                }
                if topo.subslices > 0 && topo.eu_per_subslice > 0 {
                    let eus = topo.subslices * topo.eu_per_subslice;
                    attrs.push((
                        format!("{:<LABEL_WIDTH$}", "EUs:"),
                        eus.to_string(),
                    ));
                }
            }
            if let Some(mem) = &gpu.memory {
                let size_gb = mem.size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
                attrs.push((
                    format!("{:<LABEL_WIDTH$}", "Memory:"),
                    format!("{size_gb:.1} GB shared",),
                ));
            }
            if let Some(peak) = gpu.peak_performance {
                attrs.push((
                    format!("{:<LABEL_WIDTH$}", "Peak Perf.:"),
                    format!("{:.0} GFLOPS", peak as f64 / 1_000_000_000.0),
                ));
            }
        }
        Vendor::Apple => {
            if let Some(topo) = &gpu.topology_apple {
                if topo.gpu_cores > 0 {
                    attrs.push((
                        format!("{:<LABEL_WIDTH$}", "GPU Cores:"),
                        topo.gpu_cores.to_string(),
                    ));
                }
            }
        }
    }

    attrs
}

fn memory_type_label(memtype: crate::gpu::MemoryType) -> &'static str {
    match memtype {
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
    }
}

fn print_logo_with_attrs(
    logo: &Logo,
    attributes: &[(String, String)],
    _max_label_len: usize,
    logo_color: &Color,
    text_color: &Color,
) {
    let attr_start: usize = 2;

    for (i, line) in logo.art.lines().enumerate() {
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
                                current_color = match color_num {
                                    '1' => logo_color,
                                    '2' => text_color,
                                    _ => current_color,
                                };
                            }
                        }
                        _ => {
                            output.push('$');
                            output.push(next);
                        }
                    }
                }
            } else if c != ' ' && c != '\t' {
                if first_char {
                    output.push_str(&current_color.to_ansi_fg(false));
                    first_char = false;
                }
                output.push(c);
            } else if !first_char {
                output.push_str(RESET);
                output.push(c);
                output.push_str(&current_color.to_ansi_fg(false));
                first_char = true;
            } else {
                output.push(c);
            }
        }

        print!("{output}");

        if i >= attr_start && (i - attr_start) < attributes.len() {
            let (label, value) = &attributes[i - attr_start];
            print!(
                "          {}{}{}{}{}",
                logo_color.to_ansi_fg(false),
                label,
                RESET,
                text_color.to_ansi_fg(false),
                value
            );
        }

        println!("{RESET}");
    }

    println!();
}

fn get_terminal_width() -> u32 {
    if let Ok(columns) = std::env::var("COLUMNS") {
        if let Ok(width) = columns.parse::<u32>() {
            if width > 0 {
                return width.min(MAX_TERM_WIDTH);
            }
        }
    }

    #[cfg(unix)]
    {
        use std::mem::MaybeUninit;
        use libc::{ioctl, winsize, STDOUT_FILENO, TIOCGWINSZ};

        unsafe {
            let mut ws = MaybeUninit::<winsize>::zeroed();
            if ioctl(STDOUT_FILENO, TIOCGWINSZ.into(), ws.as_mut_ptr()) == 0 {
                let ws = ws.assume_init();
                if ws.ws_col > 0 {
                    return ws.ws_col as u32;
                }
            }
        }
    }

    MAX_TERM_WIDTH
}
