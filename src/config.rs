use std::{ error, fs };

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ConfigType {
    pub color_range: f32,
    pub log_hits: bool,
    pub toggle_key: String,
    pub use_tokio: bool,
    pub check_cycles: u32,
    pub check_radius: u32,
    pub minimum_density: u32,

    pub checks: CheckToggles,
    pub target_color: RgbColor,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct CheckToggles {
    pub density: bool,
    pub radius: bool,
}

/* 
#[derive(Debug, Deserialize, Clone, Copy)]
pub struct TargetColor {
    pub min: RgbColor,
    pub max: RgbColor,
}
*/

#[derive(Debug, Deserialize, PartialEq, Eq, Clone, Copy, PartialOrd)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Ord for RgbColor {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.r < other.r && self.g < other.g && self.b < other.b {
            return std::cmp::Ordering::Less;
        }
        if self.r > other.r && self.g > other.g && self.b > other.b {
            return std::cmp::Ordering::Greater;
        }

        return std::cmp::Ordering::Equal;
    }
}

impl ConfigType {
    pub fn load() -> Result<ConfigType, Box<dyn error::Error>> {
        let file_bytes = fs::read("./config.yml")?;
        let config: ConfigType = serde_yaml::from_slice(&file_bytes)?;

        Ok(config)
    }
}
