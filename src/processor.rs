use super::cli::{Cli, WatermarkPosition};
use anyhow::Result;
use image::{DynamicImage, GenericImageView, Rgba};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale};
use std::fs;
use std::path::Path;

/// 处理单张图片的核心函数
pub fn process_image(path: &Path, cli: &Cli, font: &Font) -> Result<()> {
    println!("Processing {}...", path.display());

    let mut img = image::open(path)?;

    // 调整尺寸 (仅在指定了宽度或高度时进行)
    if cli.width.is_some() || cli.height.is_some() {
        let (width, height) = (
            cli.width.unwrap_or(img.width()),
            cli.height.unwrap_or(img.height()),
        );
        if width != img.width() || height != img.height() {
            img = img.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
        }
    }

    // 添加水印
    if let Some(text) = &cli.watermark_text {
        add_watermark(&mut img, text, font, cli.font_size, cli.watermark_position);
    }

    // 准备输出路径
    let relative_path = path.strip_prefix(&cli.input_dir)?;
    let output_path = cli.output_dir.join(relative_path);

    // 创建父目录（如果不存在）
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 保存图片，格式从扩展名推断
    img.save(&output_path)?;

    println!("Saved to {}", output_path.display());
    Ok(())
}

/// 在图片上绘制水印的辅助函数
/// 设为 `pub(crate)` 以便在 `main.rs` 的测试模块中调用
pub fn add_watermark(
    img: &mut DynamicImage, text: &str, font: &Font, font_size: u32, position: WatermarkPosition,
) {
    let scale = Scale::uniform(font_size as f32);
    let color = Rgba([255u8, 255u8, 255u8, 128u8]); // 半透明白色

    // --- 正确计算文本尺寸 ---
    let (text_width, text_height) = {
        let v_metrics = font.v_metrics(scale);
        let text_height = (v_metrics.ascent - v_metrics.descent).ceil() as u32;

        let glyphs: Vec<_> = font.layout(text, scale, rusttype::point(0.0, 0.0)).collect();
        let text_width = glyphs
            .iter()
            .rev()
            .filter_map(|g| {
                g.pixel_bounding_box()
                    .map(|bb| bb.min.x as f32 + g.unpositioned().h_metrics().advance_width)
            })
            .next()
            .unwrap_or(0.0)
            .ceil() as u32;

        (text_width, text_height)
    };

    let (img_width, img_height) = img.dimensions();
    let padding = 10;

    // 使用 saturating_sub 防止因文本过大导致整数溢出
    let safe_text_width = text_width.min(img_width.saturating_sub(padding));
    let safe_text_height = text_height.min(img_height.saturating_sub(padding));

    let (x, y) = match position {
        WatermarkPosition::Nw => (padding, padding),
        WatermarkPosition::North => ((img_width.saturating_sub(safe_text_width)) / 2, padding),
        WatermarkPosition::Ne => (img_width.saturating_sub(safe_text_width).saturating_sub(padding), padding),
        WatermarkPosition::West => (padding, (img_height.saturating_sub(safe_text_height)) / 2),
        WatermarkPosition::Center => ((img_width.saturating_sub(safe_text_width)) / 2, (img_height.saturating_sub(safe_text_height)) / 2),
        WatermarkPosition::East => (img_width.saturating_sub(safe_text_width).saturating_sub(padding), (img_height.saturating_sub(safe_text_height)) / 2),
        WatermarkPosition::Sw => (padding, img_height.saturating_sub(safe_text_height).saturating_sub(padding)),
        WatermarkPosition::South => ((img_width.saturating_sub(safe_text_width)) / 2, img_height.saturating_sub(safe_text_height).saturating_sub(padding)),
        WatermarkPosition::Se => (img_width.saturating_sub(safe_text_width).saturating_sub(padding), img_height.saturating_sub(safe_text_height).saturating_sub(padding)),
    };

    // draw_text_mut 需要的是基线的 y 坐标
    let v_metrics = font.v_metrics(scale);
    let baseline_y = y as i32 + v_metrics.ascent as i32;

    draw_text_mut(img, color, x as i32, baseline_y, scale, font, text);
}