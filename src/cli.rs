use super::errors::{ParseColorError, ParseWatermarkPositionError};
use clap::Parser;
use image::Rgba;
use std::path::PathBuf;
use std::str::FromStr;

/// 一个强大且易于使用的图片压缩和优化工具
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// 输入目录，包含需要处理的图片
    #[arg(short, long)]
    pub input_dir: PathBuf,

    /// 输出目录，用于存放处理后的图片
    #[arg(short, long)]
    pub output_dir: PathBuf,

    /// （可选）调整图片的宽度
    #[arg(long)]
    pub width: Option<u32>,

    /// （可选）调整图片的高度
    #[arg(long)]
    pub height: Option<u32>,

    /// （可选）水印文字内容
    #[arg(long)]
    pub watermark_text: Option<String>,

    /// （可选）水印位置
    #[arg(long, default_value_t = WatermarkPosition::Se, help="[possible values: nw, north, ne, west, center, east, sw, south, se]")]
    pub watermark_position: WatermarkPosition,

    /// （可选）水印文字大小 (单位: px)
    #[arg(long, default_value_t = 24)]
    pub font_size: u32,

    /// （可选）水印颜色，格式为 RRGGBB (如 'FFFFFF') 或 RRGGBBAA (如 'FFFFFF80')
    #[arg(long, default_value_t = HexColor(Rgba([255, 255, 255, 128])))]
    pub watermark_color: HexColor,
}

#[derive(Debug, Clone, Copy)]
pub struct HexColor(pub Rgba<u8>);

impl FromStr for HexColor {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.strip_prefix('#').unwrap_or(s); // 允许带 # 前缀

        if s.len() != 6 && s.len() != 8 {
            return Err(ParseColorError(s.to_string()));
        }

        let r = u8::from_str_radix(&s[0..2], 16).map_err(|_| ParseColorError(s.to_string()))?;
        let g = u8::from_str_radix(&s[2..4], 16).map_err(|_| ParseColorError(s.to_string()))?;
        let b = u8::from_str_radix(&s[4..6], 16).map_err(|_| ParseColorError(s.to_string()))?;

        // 如果提供了 alpha 通道，则解析它；否则默认为半透明 (128)
        let a = if s.len() == 8 {
            u8::from_str_radix(&s[6..8], 16).map_err(|_| ParseColorError(s.to_string()))?
        } else {
            128 // 如果只提供RRGGBB，alpha 默认设为半透明
        };

        Ok(HexColor(Rgba([r, g, b, a])))
    }
}

// 为 clap 显示默认值需要 Display trait
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

// 为 clap 显示默认值需要 Display trait
impl std::fmt::Display for WatermarkPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // 将枚举转换为小写字符串
        write!(f, "{}", format!("{:?}", self).to_lowercase())
    }
}   