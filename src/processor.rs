use anyhow::Result;
use image::{DynamicImage, GenericImage, GenericImageView, Pixel};
// use imageproc::definitions::Clamp; // 用于颜色通道计算
use rusttype::{point, Font, Scale};
use std::fs;
use std::path::Path;
use super::cli::{Cli, HexColor, WatermarkPosition};

/// 处理单张图片的核心函数
pub fn process_image(path: &Path, cli: &Cli, font: &Font) -> Result<()> {
    println!("Processing {}...", path.display());

    let mut img = image::open(path)?;
    let (original_width, original_height) = img.dimensions();

    // 智能尺寸调整逻辑
    let mut needs_resize = false;
    let (new_width, new_height) = match (cli.width, cli.height) {
        (Some(w), None) => {
            needs_resize = true;
            if original_width > 0 {
                let ratio = original_height as f32 / original_width as f32;
                let h = (w as f32 * ratio).round() as u32;
                (w, h.max(1))
            } else { (w, original_height) }
        },
        (None, Some(h)) => {
            needs_resize = true;
            if original_height > 0 {
                let ratio = original_width as f32 / original_height as f32;
                let w = (h as f32 * ratio).round() as u32;
                (w.max(1), h)
            } else { (original_width, h) }
        },
        (Some(w), Some(h)) => {
            needs_resize = true;
            (w, h)
        },
        (None, None) => (original_width, original_height),
    };

    if needs_resize && (new_width != original_width || new_height != original_height) {
        img = img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);
    }

    if let Some(text) = &cli.watermark_text {
        add_watermark(&mut img, text, font, cli.font_size, cli.watermark_position, cli.watermark_color);
    }

    let relative_path = path.strip_prefix(&cli.input_dir)?;
    let output_path = cli.output_dir.join(relative_path);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    img.save(&output_path)?;
    println!("Saved to {}", output_path.display());
    Ok(())
}

/// 根据字体、缩放和文本，计算其像素尺寸和布局偏移。
fn calculate_text_dimensions(font: &Font, scale: Scale, text: &str) -> (u32, u32, i32) {
    let v_metrics = font.v_metrics(scale);
    let text_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;

    let glyphs: Vec<_> = font.layout(text, scale, point(0.0, 0.0)).collect();

    let min_x = glyphs.iter().filter_map(|g| g.pixel_bounding_box()).map(|bb| bb.min.x).min().unwrap_or(0);
    let max_x = glyphs.iter().filter_map(|g| g.pixel_bounding_box()).map(|bb| bb.max.x).max().unwrap_or(0);

    let text_width = (max_x - min_x) as u32;

    (text_width, text_height, min_x)
}

/// 在图片上绘制水印，并能自动缩小过大的水印，同时精确定位。
///
/// 此函数通过手动遍历字形并绘制每个像素来实现像素级精确定位，
/// 从而完全绕开 imageproc 的便利性函数。
pub fn add_watermark(
    img: &mut DynamicImage, text: &str, font: &Font, font_size: u32, position: WatermarkPosition,
    color: HexColor,
) {
    let padding = 10u32;
    let (img_width, img_height) = img.dimensions();
    let watermark_color = color.0;

    let mut scale = Scale::uniform(font_size as f32);

    let max_drawable_width = img_width.saturating_sub(padding * 2);
    let max_drawable_height = img_height.saturating_sub(padding * 2);

    let (mut text_width, mut text_height, mut x_offset) = calculate_text_dimensions(font, scale, text);

    if text_width > max_drawable_width || text_height > max_drawable_height {
        let width_ratio = if text_width > 0 { max_drawable_width as f32 / text_width as f32 } else { 1.0 };
        let height_ratio = if text_height > 0 { max_drawable_height as f32 / text_height as f32 } else { 1.0 };
        let scale_factor = width_ratio.min(height_ratio);
        let new_font_size = (font_size as f32 * scale_factor).floor();
        scale = Scale::uniform(new_font_size.max(1.0));
        let (new_w, new_h, new_offset) = calculate_text_dimensions(font, scale, text);
        text_width = new_w;
        text_height = new_h;
        x_offset = new_offset;
    }

    let (target_x, target_y) = {
        let iw = img_width;
        let ih = img_height;
        let tw = text_width;
        let th = text_height;

        match position {
            WatermarkPosition::Nw => (padding, padding),
            WatermarkPosition::North => ((iw - tw) / 2, padding),
            WatermarkPosition::Ne => (iw - tw - padding, padding),
            WatermarkPosition::West => (padding, (ih - th) / 2),
            WatermarkPosition::Center => ((iw - tw) / 2, (ih - th) / 2),
            WatermarkPosition::East => (iw - tw - padding, (ih - th) / 2),
            WatermarkPosition::Sw => (padding, ih - th - padding),
            WatermarkPosition::South => ((iw - tw) / 2, ih - th - padding),
            WatermarkPosition::Se => (iw - tw - padding, ih - th - padding),
        }
    };

    let v_metrics = font.v_metrics(scale);
    let origin_x = target_x as i32 - x_offset;
    let origin_y = target_y as i32 + v_metrics.ascent as i32;

    let glyphs: Vec<_> = font.layout(text, scale, point(origin_x as f32, origin_y as f32)).collect();

    // 手动遍历并绘制每个字形的每个像素
    for g in glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            g.draw(|x, y, v| {
                // v 是 [0, 1] 范围的像素覆盖率
                if v > 0.0 {
                    let px = (bb.min.x + x as i32) as u32;
                    let py = (bb.min.y + y as i32) as u32;

                    // 确保像素在图片范围内
                    if px < img_width && py < img_height {
                        // 创建一个基于覆盖率的、半透明的白色像素
                        let weighted_color = {
                            let mut color = watermark_color;
                            color.0[3] = (color.0[3] as f32 * v) as u8;
                            color
                        };

                        // 获取背景像素并进行 Alpha 混合
                        let mut background_pixel = img.get_pixel(px, py);
                        background_pixel.blend(&weighted_color);

                        // 将混合后的像素画回图片
                        img.put_pixel(px, py, background_pixel);
                    }
                }
            });
        }
    }
}