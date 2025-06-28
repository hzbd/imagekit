use super::cli::{Cli, HexColor, WatermarkPosition};
use anyhow::Result;
use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{DynamicImage, GenericImage, GenericImageView, ImageEncoder, Pixel};
use rusttype::{point, Font, PositionedGlyph, Scale};
use std::fs;
use std::io::BufWriter;
use std::path::Path;

/// 处理单张图片的核心函数
pub fn process_image(path: &Path, cli: &Cli, fonts: &[Font<'static>]) -> Result<()> {
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
        (Some(w), Some(h)) => { needs_resize = true; (w, h) },
        (None, None) => (original_width, original_height),
    };

    if needs_resize && (new_width != original_width || new_height != original_height) {
        img = img.resize_exact(new_width, new_height, image::imageops::FilterType::Lanczos3);
    }

    if let Some(text) = &cli.watermark_text {
        add_watermark(&mut img, text, fonts, cli.font_size, cli.watermark_position, cli.watermark_color);
    }

    let relative_path = path.strip_prefix(&cli.input_dir)?;
    let output_path = cli.output_dir.join(relative_path);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // 图片保存逻辑
    let file = fs::File::create(&output_path)?;
    let writer = BufWriter::new(file);
    let ext = output_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => {
            let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(writer, cli.quality);
            encoder.write_image(img.as_bytes(), img.width(), img.height(), img.color())?;
        },
        "png" => {
            let compression = match cli.quality {
                100 => CompressionType::Best,
                1..=50 => CompressionType::Fast,
                _ => CompressionType::Default,
            };
            let encoder = PngEncoder::new_with_quality(writer, compression, FilterType::Sub);
            encoder.write_image(img.as_bytes(), img.width(), img.height(), img.color())?;
        },
        _ => { img.save(&output_path)?; }
    }

    println!("Saved to {}", output_path.display());
    Ok(())
}

/// 根据字体列表、缩放和文本，计算并布局字形，支持字体回退。
/// 返回一个包含所有已定位字形的向量，以及整个文本的精确像素边界框信息。
fn layout_text<'a>(
    text: &str,
    scale: Scale,
    fonts: &'a [Font<'static>],
) -> (Vec<PositionedGlyph<'a>>, u32, u32, i32) {
    if fonts.is_empty() {
        return (vec![], 0, 0, 0);
    }
    let primary_font = &fonts[0];

    let mut glyphs = Vec::new();
    let v_metrics = primary_font.v_metrics(scale);
    let base_ascent = v_metrics.ascent;
    let mut caret = 0.0;
    let mut last_glyph_id = None;

    for ch in text.chars() {
        let (font_used, glyph) = fonts
            .iter()
            .find_map(|f| {
                let g = f.glyph(ch);
                if g.id() != rusttype::GlyphId(0) { Some((f, g)) } else { None }
            })
            .unwrap_or_else(|| (primary_font, primary_font.glyph('\u{FFFD}')));

        let scaled_glyph = glyph.scaled(scale);
        if let Some(id) = last_glyph_id {
            caret += font_used.pair_kerning(scale, id, scaled_glyph.id());
        }

        let positioned_glyph = scaled_glyph.positioned(point(caret, base_ascent));
        caret += positioned_glyph.unpositioned().h_metrics().advance_width;
        last_glyph_id = Some(positioned_glyph.id());

        glyphs.push(positioned_glyph);
    }

    // 在所有字形都布局好之后，再计算整体的像素边界框
    let (min_x, max_x, min_y, max_y) = glyphs
        .iter()
        .filter_map(|g| g.pixel_bounding_box())
        .fold((i32::MAX, i32::MIN, i32::MAX, i32::MIN), |(min_x, max_x, min_y, max_y), bb| {
            (min_x.min(bb.min.x), max_x.max(bb.max.x), min_y.min(bb.min.y), max_y.max(bb.max.y))
        });

    let text_width = if min_x <= max_x { (max_x - min_x) as u32 } else { 0 };
    let text_height = if min_y <= max_y { (max_y - min_y) as u32 } else { 0 };
    let final_min_x = if min_x == i32::MAX { 0 } else { min_x };

    (glyphs, text_width, text_height, final_min_x)
}

/// 在图片上绘制水印，并能自动缩小过大的水印，同时精确定位。
/// 此版本支持 CJK 字符回退。
pub fn add_watermark(
    img: &mut DynamicImage,
    text: &str,
    fonts: &[Font<'static>],
    font_size: u32,
    position: WatermarkPosition,
    color: HexColor,
) {
    if fonts.is_empty() { return; }

    let padding = 10u32;
    let (img_width, img_height) = img.dimensions();
    let watermark_color = color.0;

    let mut scale = Scale::uniform(font_size as f32);

    let max_drawable_width = img_width.saturating_sub(padding * 2);
    let max_drawable_height = img_height.saturating_sub(padding * 2);

    let (_, text_width, text_height, _) = layout_text(text, scale, fonts);

    if text_width > max_drawable_width || text_height > max_drawable_height {
        let width_ratio = if text_width > 0 { max_drawable_width as f32 / text_width as f32 } else { 1.0 };
        let height_ratio = if text_height > 0 { max_drawable_height as f32 / text_height as f32 } else { 1.0 };
        let scale_factor = width_ratio.min(height_ratio);
        let new_font_size = (font_size as f32 * scale_factor).floor();
        scale = Scale::uniform(new_font_size.max(1.0));
    }

    let (glyphs, text_width, text_height, x_offset) = layout_text(text, scale, fonts);

    let (target_x, target_y) = {
        let iw = img_width; let ih = img_height;
        let tw = text_width; let th = text_height;
        match position {
            WatermarkPosition::Nw => (padding, padding),
            WatermarkPosition::North => ((iw.saturating_sub(tw)) / 2, padding),
            WatermarkPosition::Ne => (iw.saturating_sub(tw).saturating_sub(padding), padding),
            WatermarkPosition::West => (padding, (ih.saturating_sub(th)) / 2),
            WatermarkPosition::Center => ((iw.saturating_sub(tw)) / 2, (ih.saturating_sub(th)) / 2),
            WatermarkPosition::East => (iw.saturating_sub(tw).saturating_sub(padding), (ih.saturating_sub(th)) / 2),
            WatermarkPosition::Sw => (padding, ih.saturating_sub(th).saturating_sub(padding)),
            WatermarkPosition::South => ((iw.saturating_sub(tw)) / 2, ih.saturating_sub(th).saturating_sub(padding)),
            WatermarkPosition::Se => (iw.saturating_sub(tw).saturating_sub(padding), ih.saturating_sub(th).saturating_sub(padding)),
        }
    };

    let final_x_offset = target_x as i32 - x_offset;
    let final_y_offset = target_y as i32;

    for g in &glyphs {
        if let Some(bb) = g.pixel_bounding_box() {
            let bb_x = bb.min.x + final_x_offset;
            let bb_y = bb.min.y + final_y_offset;
            g.draw(|x, y, v| {
                if v > 0.0 {
                    let px = bb_x + x as i32;
                    let py = bb_y + y as i32;
                    if px >= 0 && py >= 0 && (px as u32) < img_width && (py as u32) < img_height {
                        let mut weighted_color = watermark_color;
                        weighted_color.0[3] = (weighted_color.0[3] as f32 * v) as u8;
                        let mut background_pixel = img.get_pixel(px as u32, py as u32);
                        background_pixel.blend(&weighted_color);
                        img.put_pixel(px as u32, py as u32, background_pixel);
                    }
                }
            });
        }
    }
}