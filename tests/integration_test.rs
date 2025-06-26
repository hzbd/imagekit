use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, Rgba};
use tempfile::tempdir;

// 导入我们库中的公共项
use imagekit::{
    assets::Asset,
    cli::{Cli, HexColor, WatermarkPosition}, // 确保导入 HexColor
    processor::add_watermark,
    run,
};

/// 测试水印位置的字符串解析是否正确。
#[test]
fn test_watermark_position_from_str() {
    use std::str::FromStr;
    assert_eq!(WatermarkPosition::from_str("nw").unwrap(), WatermarkPosition::Nw);
    assert!(WatermarkPosition::from_str("top-left").is_err());
}

/// 测试十六进制颜色码的解析。
#[test]
fn test_hex_color_parsing() {
    use std::str::FromStr;
    // 测试带 #
    let color = HexColor::from_str("#FF000080").unwrap();
    assert_eq!(color.0, Rgba([255, 0, 0, 128]));

    // 测试不带 #
    let color = HexColor::from_str("00FF00").unwrap();
    assert_eq!(color.0, Rgba([0, 255, 0, 128])); // 默认 alpha

    // 测试大小写不敏感
    let color = HexColor::from_str("0000ffAA").unwrap();
    assert_eq!(color.0, Rgba([0, 0, 255, 170]));

    // 测试错误格式
    assert!(HexColor::from_str("12345").is_err());
    assert!(HexColor::from_str("GGFFFF").is_err());
}


/// 单元测试：验证 `add_watermark` 函数是否确实修改了图片。
#[test]
fn test_add_watermark_logic() -> Result<()> {
    let mut img = DynamicImage::ImageRgba8(
        image::RgbaImage::from_pixel(200, 200, Rgba([0, 0, 0, 255]))
    );
    let original_img_bytes = img.as_bytes().to_vec();

    let font_data = Asset::get("Roboto-Regular.ttf").expect("Font not found for testing");
    let font = rusttype::Font::try_from_bytes(font_data.data.as_ref())
        .context("Failed to parse font from embedded data in test")?;

    // --- 修正点：添加颜色参数 ---
    let default_color = HexColor(Rgba([255, 255, 255, 128]));
    add_watermark(&mut img, "Test", &font, 20, WatermarkPosition::Se, default_color);

    let watermarked_img_bytes = img.as_bytes().to_vec();
    assert_ne!(
        original_img_bytes,
        watermarked_img_bytes,
        "Image content should have changed after applying watermark"
    );

    Ok(())
}

/// 集成测试：模拟一次完整的命令行运行，包括调整尺寸和添加水印。
#[test]
fn test_full_run_with_resize_and_watermark() -> Result<()> {
    let input_dir = tempdir()?;
    let output_dir = tempdir()?;

    let test_image_path = input_dir.path().join("test.png");
    let initial_image = image::RgbaImage::from_pixel(200, 200, Rgba([0, 0, 0, 0]));
    initial_image.save(&test_image_path)?;

    let original_bytes = std::fs::read(&test_image_path)?;

    // --- 修正点：为 Cli 实例添加 watermark_color ---
    let cli = Cli {
        input_dir: input_dir.path().to_path_buf(),
        output_dir: output_dir.path().to_path_buf(),
        width: Some(100),
        height: Some(80),
        watermark_text: Some("Integration Test".to_string()),
        watermark_position: WatermarkPosition::Center,
        font_size: 16,
        watermark_color: HexColor(Rgba([255, 255, 255, 128])), // 添加默认颜色
    };

    run(cli)?;

    let output_image_path = output_dir.path().join("test.png");
    assert!(output_image_path.exists(), "Output image was not created");

    let processed_bytes = std::fs::read(&output_image_path)?;
    assert_ne!(original_bytes, processed_bytes, "Image content did not change after processing");

    let output_img = image::open(&output_image_path)?;
    assert_eq!(output_img.dimensions(), (100, 80), "Image was not resized correctly");

    Ok(())
}

/// 测试：当只提供宽度时，是否能按比例缩放。
#[test]
fn test_run_proportional_resize_by_width() -> Result<()> {
    let input_dir = tempdir()?;
    let output_dir = tempdir()?;

    let test_image_path = input_dir.path().join("tall_image.png");
    image::RgbaImage::new(200, 400).save(&test_image_path)?;

    // --- 修正点：为 Cli 实例添加 watermark_color ---
    let cli = Cli {
        input_dir: input_dir.path().to_path_buf(),
        output_dir: output_dir.path().to_path_buf(),
        width: Some(100),
        height: None,
        watermark_text: None,
        watermark_position: WatermarkPosition::Se,
        font_size: 24,
        watermark_color: HexColor(Rgba([255, 255, 255, 128])), // 添加默认颜色
    };

    run(cli)?;

    let output_image_path = output_dir.path().join("tall_image.png");
    assert!(output_image_path.exists());
    let output_img = image::open(&output_image_path)?;

    assert_eq!(output_img.dimensions(), (100, 200));

    Ok(())
}

/// 测试：当只提供高度时，是否能按比例缩放。
#[test]
fn test_run_proportional_resize_by_height() -> Result<()> {
    let input_dir = tempdir()?;
    let output_dir = tempdir()?;

    let test_image_path = input_dir.path().join("wide_image.png");
    image::RgbaImage::new(400, 200).save(&test_image_path)?;

    // --- 修正点：为 Cli 实例添加 watermark_color ---
    let cli = Cli {
        input_dir: input_dir.path().to_path_buf(),
        output_dir: output_dir.path().to_path_buf(),
        width: None,
        height: Some(100),
        watermark_text: None,
        watermark_position: WatermarkPosition::Se,
        font_size: 24,
        watermark_color: HexColor(Rgba([255, 255, 255, 128])), // 添加默认颜色
    };

    run(cli)?;

    let output_image_path = output_dir.path().join("wide_image.png");
    assert!(output_image_path.exists());
    let output_img = image::open(&output_image_path)?;

    assert_eq!(output_img.dimensions(), (200, 100));

    Ok(())
}

/// 测试：当请求的字体大小对于图片来说过大时，水印应能被自动缩小以适应图片。
#[test]
fn test_watermark_autoscales_down_when_too_large() -> Result<()> {
    let mut img = DynamicImage::ImageRgba8(
        image::RgbaImage::from_pixel(100, 50, Rgba([0, 0, 0, 255]))
    );
    let original_img_bytes = img.as_bytes().to_vec();

    let font_data = Asset::get("Roboto-Regular.ttf").expect("Font not found for testing");
    let font = rusttype::Font::try_from_bytes(font_data.data.as_ref())
        .context("Failed to parse font from embedded data in test")?;

    // --- 修正点：添加颜色参数 ---
    add_watermark(
        &mut img,
        "This text is definitely too long",
        &font,
        40,
        WatermarkPosition::Center,
        HexColor(Rgba([255, 255, 255, 128])), // 添加默认颜色
    );

    let processed_img_bytes = img.as_bytes().to_vec();
    assert_ne!(original_img_bytes, processed_img_bytes, "Watermark should have been applied");

    let black = Rgba([0, 0, 0, 255]);
    assert_eq!(img.get_pixel(0, 0), black, "Top-left corner should not be touched by watermark");
    assert_eq!(img.get_pixel(99, 0), black, "Top-right corner should not be touched by watermark");
    assert_eq!(img.get_pixel(0, 49), black, "Bottom-left corner should not be touched by watermark");
    assert_eq!(img.get_pixel(99, 49), black, "Bottom-right corner should not be touched by watermark");

    Ok(())
}