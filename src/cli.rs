use super::errors::{ParseColorError, ParseWatermarkPositionError};
use clap::Parser;
use image::Rgba;
use std::path::PathBuf;
use std::str::FromStr;

/// A powerful and easy-to-use image compression and optimization tool.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The input directory containing images to process.
    #[arg(short, long)]
    pub input_dir: PathBuf,

    /// The output directory for the processed images.
    #[arg(short, long)]
    pub output_dir: PathBuf,

    /// (Optional) The target width for resizing the image.
    #[arg(long)]
    pub width: Option<u32>,

    /// (Optional) The target height for resizing the image.
    #[arg(long)]
    pub height: Option<u32>,

    /// (Optional) The text for the watermark.
    #[arg(long)]
    pub watermark_text: Option<String>,

    /// (Optional) The position of the watermark.
    #[arg(long, default_value_t = WatermarkPosition::Se, help="[possible values: nw, north, ne, west, center, east, sw, south, se]")]
    pub watermark_position: WatermarkPosition,

    /// (Optional) The font size for the watermark text (in pixels).
    #[arg(long, default_value_t = 24)]
    pub font_size: u32,

    /// (Optional) The color of the watermark, in RRGGBB (e.g., 'FFFFFF') or RRGGBBAA (e.g., 'FFFFFF80') hex format.
    #[arg(long, default_value_t = HexColor(Rgba([255, 255, 255, 128])))]
    pub watermark_color: HexColor,

    /// (Optional) The output quality (1-100). For JPEG, this directly affects the compression ratio. For PNG, it affects compression time and file size.
    #[arg(short, long, default_value_t = 85, value_parser = clap::value_parser!(u8).range(1..=100))]
    pub quality: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct HexColor(pub Rgba<u8>);

impl FromStr for HexColor {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix('#').unwrap_or(s); // Allow an optional '#' prefix

        if s.len() != 6 && s.len() != 8 {
            return Err(ParseColorError(s.to_string()));
        }

        let r = u8::from_str_radix(&s[0..2], 16).map_err(|_| ParseColorError(s.to_string()))?;
        let g = u8::from_str_radix(&s[2..4], 16).map_err(|_| ParseColorError(s.to_string()))?;
        let b = u8::from_str_radix(&s[4..6], 16).map_err(|_| ParseColorError(s.to_string()))?;

        // If an alpha channel is provided, parse it; otherwise, default to semi-transparent (128).
        let a = if s.len() == 8 {
            u8::from_str_radix(&s[6..8], 16).map_err(|_| ParseColorError(s.to_string()))?
        } else {
            128 // Default alpha to semi-transparent if only RRGGBB is provided.
        };

        Ok(HexColor(Rgba([r, g, b, a])))
    }
}

// The Display trait is required for clap to show the default value.
impl std::fmt::Display for HexColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02x}{:02x}{:02x}{:02x}", self.0[0], self.0[1], self.0[2], self.0[3])
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WatermarkPosition {
    Nw, North, Ne, West, Center, East, Sw, South, Se,
}

impl FromStr for WatermarkPosition {
    type Err = ParseWatermarkPositionError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "nw" => Ok(Self::Nw), "north" => Ok(Self::North), "ne" => Ok(Self::Ne),
            "west" => Ok(Self::West), "center" => Ok(Self::Center), "east" => Ok(Self::East),
            "sw" => Ok(Self::Sw), "south" => Ok(Self::South), "se" => Ok(Self::Se),
            _ => Err(ParseWatermarkPositionError(s.to_string())),
        }
    }
}

// The Display trait is required for clap to show the default value.
impl std::fmt::Display for WatermarkPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Convert the enum variant to a lowercase string.
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}