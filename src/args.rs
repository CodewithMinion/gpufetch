use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "gpufetch")]
#[command(author = "NLP-Core-Team")]
#[command(version = "0.1.0")]
#[command(about = "Simple yet fancy GPU architecture fetching tool", long_about = None)]
pub struct Args {
    /// Set the color scheme
    #[arg(short, long, value_name = "COLOR")]
    pub color: Option<String>,
    
    /// Select the GPU to use (default: 0). Use 'all' for all GPUs
    #[arg(short, long, value_name = "GPU", default_value = "0")]
    pub gpu: String,
    
    /// List the available GPUs in the system
    #[arg(short = 'l', long = "list-gpus")]
    pub list_gpus: bool,
    
    /// Show the short version of the logo
    #[arg(long = "logo-short")]
    pub logo_short: bool,
    
    /// Show the long version of the logo
    #[arg(long = "logo-long")]
    pub logo_long: bool,
    
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
    
    /// Print help
    #[arg(short, long)]
    pub help: bool,
    
    /// Print version
    #[arg(short = 'V', long)]
    pub version: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorScheme {
    Nvidia,
    Amd,
    Intel,
    Custom(Vec<u8>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Style {
    Empty,
    Fancy,
    Retro,
    Legacy,
}

impl Default for Style {
    fn default() -> Self {
        Style::Fancy
    }
}

pub fn print_help() {
    println!(
        r#"Usage: gpufetch [OPTIONS]

Simple yet fancy GPU architecture fetching tool

Options:
  -c, --color <COLOR>        Set the color scheme
  -g, --gpu <GPU>            Select the GPU to use (default: 0)
  -l, --list-gpus            List the available GPUs
      --logo-short           Show short logo
      --logo-long            Show long logo
  -v, --verbose              Verbose output
  -h, --help                 Print help
  -V, --version              Print version

COLORS:
  Valid color schemes: intel, amd, nvidia
  Custom colors: R,G,B:R,G,B:R,G,B:R,G,B (5 RGB values)

EXAMPLES:
  gpufetch --color intel
  gpufetch --color 239,90,45:210,200,200:0,0,0:100,200,45:0,200,200"#
    );
}
