use super::errors::{ParseColorError, ParseWatermarkPositionError};
use clap::Parser;
use image::Rgba;
use std::path::PathBuf;
use std::str::FromStr;
use clap::ValueEnum;
use image::ImageFormat;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long)]
    pub input_dir: PathBuf,

    #[arg(short, long)]
    pub output_dir: PathBuf,

    #[arg(long)]
    pub width: Option<u32>,

    #[arg(long)]
    pub height: Option<u32>,

    #[arg(long)]
    pub watermark_text: Option<String>,

    #[arg(long, default_value_t = WatermarkPosition::Se, help="[possible values: nw, north, ne, west, center, east, sw, south, se]")]
    pub watermark_position: WatermarkPosition,

    #[arg(long, default_value_t = 24)]
    pub font_size: u32,

    #[arg(long, default_value_t = HexColor(Rgba([255, 255, 255, 128])))]
    pub watermark_color: HexColor,

    #[arg(short, long, default_value_t = 85, value_parser = clap::value_parser!(u8).range(1..=100))]
    pub quality: u8,

    #[arg(long, value_enum, help = "Specify the output image format")]
    pub output_format: Option<OutputFormat>,
}

#[derive(Debug, Clone, Copy)]
pub struct HexColor(pub Rgba<u8>);

impl FromStr for HexColor {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix('#').unwrap_or(s);

        if s.len() != 6 && s.len() != 8 {
            return Err(ParseColorError(s.to_string()));
        }

        let r = u8::from_str_radix(&s[0..2], 16).map_err(|_| ParseColorError(s.to_string()))?;
        let g = u8::from_str_radix(&s[2..4], 16).map_err(|_| ParseColorError(s.to_string()))?;
        let b = u8::from_str_radix(&s[4..6], 16).map_err(|_| ParseColorError(s.to_string()))?;

        let a = if s.len() == 8 {
            u8::from_str_radix(&s[6..8], 16).map_err(|_| ParseColorError(s.to_string()))?
        } else {
            128
        };

        Ok(HexColor(Rgba([r, g, b, a])))
    }
}

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

impl std::fmt::Display for WatermarkPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}

#[derive(Clone, Debug, ValueEnum)]
pub enum OutputFormat {
    Jpg,
    Png,
    Webp,
    Gif,
    Bmp,
}

impl From<OutputFormat> for ImageFormat {
    fn from(format: OutputFormat) -> Self {
        match format {
            OutputFormat::Jpg => ImageFormat::Jpeg,
            OutputFormat::Png => ImageFormat::Png,
            OutputFormat::Webp => ImageFormat::WebP,
            OutputFormat::Gif => ImageFormat::Gif,
            OutputFormat::Bmp => ImageFormat::Bmp,
        }
    }
}
