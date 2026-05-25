#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    
    // NVIDIA - Green for labels
    pub const fn nvidia_logo() -> Self {
        Self::new(76, 153, 0)  // Green
    }
    
    pub const fn nvidia_text() -> Self {
        Self::new(255, 255, 255)  // White for values
    }
    
    // AMD - Red for labels
    pub const fn amd_logo() -> Self {
        Self::new(200, 30, 30)  // Red
    }
    
    pub const fn amd_text() -> Self {
        Self::new(255, 255, 255)  // White for values
    }
    
    // Intel - Cyan/Teal for labels
    pub const fn intel_logo() -> Self {
        Self::new(0, 180, 215)  // Cyan/Teal
    }
    
    pub const fn intel_text() -> Self {
        Self::new(255, 255, 255)  // White for values
    }
    
    pub fn to_ansi_fg(&self, bold: bool) -> String {
        if bold {
            format!("\x1b[1m\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
        } else {
            format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
        }
    }
    
    pub fn to_ansi_bg(&self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.r, self.g, self.b)
    }
}

pub const RESET: &str = "\x1b[0m";

pub mod ansi {
    pub const BLACK: &str = "\x1b[30m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";
    
    pub const BOLD: &str = "\x1b[1m";
    pub const RESET: &str = "\x1b[0m";
}

pub fn parse_color_string(s: &str) -> Result<Vec<Color>, String> {
    let parts: Vec<&str> = s.split(':').collect();
    let mut colors = Vec::new();
    
    for part in parts {
        let nums: Vec<&str> = part.split(',').collect();
        if nums.len() != 3 {
            return Err(format!("Invalid color format: {}", part));
        }
        
        let r = nums[0].trim().parse::<u8>()
            .map_err(|_| format!("Invalid RGB value: {}", nums[0]))?;
        let g = nums[1].trim().parse::<u8>()
            .map_err(|_| format!("Invalid RGB value: {}", nums[1]))?;
        let b = nums[2].trim().parse::<u8>()
            .map_err(|_| format!("Invalid RGB value: {}", nums[2]))?;
        
        colors.push(Color::new(r, g, b));
    }
    
    Ok(colors)
}

pub fn parse_color_scheme(s: &str) -> Result<(Vec<Color>, Vec<Color>), String> {
    match s.to_lowercase().as_str() {
        "nvidia" => Ok((
            vec![Color::nvidia_logo(), Color::nvidia_logo()],
            vec![Color::nvidia_text(), Color::nvidia_logo()],
        )),
        "amd" => Ok((
            vec![Color::amd_logo(), Color::amd_logo()],
            vec![Color::amd_text(), Color::amd_text()],
        )),
        "intel" => Ok((
            vec![Color::intel_logo()],
            vec![Color::intel_text(), Color::intel_text()],
        )),
        _ => Err(format!("Unknown color scheme: {}", s)),
    }
}
