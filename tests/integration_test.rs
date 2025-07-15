use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, Rgba};
use tempfile::tempdir;
use std::fs;

// Import public items from our library.
use imagekit::{
    assets::Asset,
    cli::{Cli, HexColor, WatermarkPosition},
    processor::add_watermark,
    run,
};
// Import `Font` to be able to create it in tests.
use rusttype::Font;

/// Helper function to load a list of fonts for testing.
/// It solves lifetime issues by taking ownership of the data.
fn load_test_fonts() -> Result<Vec<Font<'static>>> {
    // 1. Load primary font data and take ownership.
    let primary_font_data = Asset::get("Roboto-Regular.ttf")
        .context("Test setup failed: Could not find 'Roboto-Regular.ttf'")?;
    let primary_font_vec: Vec<u8> = primary_font_data.data.into_owned();

    // 2. Load fallback CJK font data and take ownership.
    let fallback_font_data = Asset::get("SourceHanSansSC-Regular.otf")
        .context("Test setup failed: Could not find 'SourceHanSansSC-Regular.otf'")?;
    let fallback_font_vec: Vec<u8> = fallback_font_data.data.into_owned();

    // 3. Create Font objects from our owned data.
    let primary_font = Font::try_from_vec(primary_font_vec)
        .context("Test setup failed: Could not parse 'Roboto-Regular.ttf'")?;
    let fallback_font = Font::try_from_vec(fallback_font_vec)
        .context("Test setup failed: Could not parse 'SourceHanSansSC-Regular.otf'")?;

    Ok(vec![primary_font, fallback_font])
}


/// Tests that watermark position parsing from a string is correct.
#[test]
fn test_watermark_position_from_str() {
    use std::str::FromStr;
    assert_eq!(WatermarkPosition::from_str("nw").unwrap(), WatermarkPosition::Nw);
    assert!(WatermarkPosition::from_str("top-left").is_err());
}

/// Tests hex color code parsing.
#[test]
fn test_hex_color_parsing() {
    use std::str::FromStr;
    let color = HexColor::from_str("#FF000080").unwrap();
    assert_eq!(color.0, Rgba([255, 0, 0, 128]));
    let color = HexColor::from_str("00FF00").unwrap();
    assert_eq!(color.0, Rgba([0, 255, 0, 128]));
    let color = HexColor::from_str("0000ffAA").unwrap();
    assert_eq!(color.0, Rgba([0, 0, 255, 170]));
    assert!(HexColor::from_str("12345").is_err());
    assert!(HexColor::from_str("GGFFFF").is_err());
}

/// Unit test: Verifies that the `add_watermark` function actually modifies the image.
#[test]
fn test_add_watermark_logic() -> Result<()> {
    let mut img = DynamicImage::ImageRgba8(
        image::RgbaImage::from_pixel(200, 200, Rgba([0, 0, 0, 255]))
    );
    let original_img_bytes = img.as_bytes().to_vec();

    let fonts = load_test_fonts()?;

    let default_color = HexColor(Rgba([255, 255, 255, 128]));
    add_watermark(&mut img, "Test", &fonts, 20, WatermarkPosition::Se, default_color);

    let watermarked_img_bytes = img.as_bytes().to_vec();
    assert_ne!(
        original_img_bytes,
        watermarked_img_bytes,
        "Image content should have changed after applying watermark"
    );

    Ok(())
}

/// Integration test: Simulates a full command-line run, including resizing and watermarking.
#[test]
fn test_full_run_with_resize_and_watermark() -> Result<()> {
    let input_dir = tempdir()?;
    let output_dir = tempdir()?;

    let test_image_path = input_dir.path().join("test.png");
    image::RgbaImage::from_pixel(200, 200, Rgba([0, 0, 0, 0])).save(&test_image_path)?;

    let original_bytes = fs::read(&test_image_path)?;

    let cli = Cli {
        input_dir: input_dir.path().to_path_buf(),
        output_dir: output_dir.path().to_path_buf(),
        width: Some(100),
        height: Some(80),
        watermark_text: Some("Integration Test".to_string()),
        watermark_position: WatermarkPosition::Center,
        font_size: 16,
        watermark_color: HexColor(Rgba([255, 255, 255, 128])),
        quality: 85,
        output_format: None, // FIX: Added missing field.
    };

    run(cli)?;

    let output_image_path = output_dir.path().join("test.png");
    assert!(output_image_path.exists(), "Output image was not created");

    let processed_bytes = fs::read(&output_image_path)?;
    assert_ne!(original_bytes, processed_bytes, "Image content did not change after processing");

    let output_img = image::open(&output_image_path)?;
    assert_eq!(output_img.dimensions(), (100, 80), "Image was not resized correctly");

    Ok(())
}

/// Test: Ensures proportional scaling works when only width is provided.
#[test]
fn test_run_proportional_resize_by_width() -> Result<()> {
    let input_dir = tempdir()?;
    let output_dir = tempdir()?;

    let test_image_path = input_dir.path().join("tall_image.png");
    image::RgbaImage::new(200, 400).save(&test_image_path)?;

    let cli = Cli {
        input_dir: input_dir.path().to_path_buf(),
        output_dir: output_dir.path().to_path_buf(),
        width: Some(100),
        height: None,
        watermark_text: None,
        watermark_position: WatermarkPosition::Se,
        font_size: 24,
        watermark_color: HexColor(Rgba([255, 255, 255, 128])),
        quality: 85,
        output_format: None, // FIX: Added missing field.
    };

    run(cli)?;

    let output_image_path = output_dir.path().join("tall_image.png");
    assert!(output_image_path.exists());
    let output_img = image::open(&output_image_path)?;

    assert_eq!(output_img.dimensions(), (100, 200));

    Ok(())
}

/// Test: Ensures proportional scaling works when only height is provided.
#[test]
fn test_run_proportional_resize_by_height() -> Result<()> {
    let input_dir = tempdir()?;
    let output_dir = tempdir()?;

    let test_image_path = input_dir.path().join("wide_image.png");
    image::RgbaImage::new(400, 200).save(&test_image_path)?;

    let cli = Cli {
        input_dir: input_dir.path().to_path_buf(),
        output_dir: output_dir.path().to_path_buf(),
        width: None,
        height: Some(100),
        watermark_text: None,
        watermark_position: WatermarkPosition::Se,
        font_size: 24,
        watermark_color: HexColor(Rgba([255, 255, 255, 128])),
        quality: 85,
        output_format: None, // FIX: Added missing field.
    };

    run(cli)?;

    let output_image_path = output_dir.path().join("wide_image.png");
    assert!(output_image_path.exists());
    let output_img = image::open(&output_image_path)?;

    assert_eq!(output_img.dimensions(), (200, 100));

    Ok(())
}

/// Test: Ensures the watermark autoscales down when it's too large for the image.
#[test]
fn test_watermark_autoscales_down_when_too_large() -> Result<()> {
    let mut img = DynamicImage::ImageRgba8(
        image::RgbaImage::from_pixel(100, 50, Rgba([0, 0, 0, 255]))
    );
    let original_img_bytes = img.as_bytes().to_vec();

    let fonts = load_test_fonts()?;

    add_watermark(
        &mut img,
        "This text is definitely too long",
        &fonts,
        40,
        WatermarkPosition::Center,
        HexColor(Rgba([255, 255, 255, 128])),
    );

    let processed_img_bytes = img.as_bytes().to_vec();
    assert_ne!(original_img_bytes, processed_img_bytes, "Watermark should have been applied");

    let black = Rgba([0, 0, 0, 255]);
    assert_eq!(img.get_pixel(0, 0), black);
    assert_eq!(img.get_pixel(99, 0), black);
    assert_eq!(img.get_pixel(0, 49), black);
    assert_eq!(img.get_pixel(99, 49), black);

    Ok(())
}

/// Verifies that the quality parameter affects the final file size.
#[test]
fn test_quality_options_affect_file_size() -> Result<()> {
    let input_dir = tempdir()?;
    let test_image_path = input_dir.path().join("quality_test.jpg");
    image::RgbImage::new(200, 200).save(&test_image_path)?;

    // 1. Save with low quality.
    let low_q_output_dir = tempdir()?;
    let cli_low = Cli {
        input_dir: input_dir.path().to_path_buf(),
        output_dir: low_q_output_dir.path().to_path_buf(),
        quality: 10,
        width: None, height: None, watermark_text: None,
        watermark_position: WatermarkPosition::Se, font_size: 24,
        watermark_color: HexColor(Rgba([255,255,255,128])),
        output_format: None, // FIX: Added missing field.
    };
    run(cli_low)?;
    let low_q_size = fs::metadata(low_q_output_dir.path().join("quality_test.jpg"))?.len();

    // 2. Save with high quality.
    let high_q_output_dir = tempdir()?;
    let cli_high = Cli {
        input_dir: input_dir.path().to_path_buf(),
        output_dir: high_q_output_dir.path().to_path_buf(),
        quality: 100,
        width: None, height: None, watermark_text: None,
        watermark_position: WatermarkPosition::Se, font_size: 24,
        watermark_color: HexColor(Rgba([255,255,255,128])),
        output_format: None, // FIX: Added missing field.
    };
    run(cli_high)?;
    let high_q_size = fs::metadata(high_q_output_dir.path().join("quality_test.jpg"))?.len();

    println!("Low-Q: {}, High-Q: {}", low_q_size, high_q_size);
    assert!(low_q_size < high_q_size, "Low quality JPEG should be smaller than high quality JPEG");

    Ok(())
}

// Verifies CJK character support.
#[test]
fn test_cjk_watermark_support() -> Result<()> {
    // 1. Prepare an image with a known background.
    let mut img = DynamicImage::ImageRgba8(
        image::RgbaImage::from_pixel(300, 100, Rgba([255, 255, 255, 255]))
    );
    // 2. Clone the image byte data before processing.
    let original_img_bytes = img.as_bytes().to_vec();

    let fonts = load_test_fonts()?;

    // 3. Perform the watermarking operation.
    add_watermark(
        &mut img,
        "测试 Test 123 テスト", // Mixed Chinese, English, numbers, and Japanese.
        &fonts,
        30,
        WatermarkPosition::Center,
        HexColor(Rgba([0, 0, 0, 128])), // Semi-transparent black.
    );

    // 4. Get the image byte data after processing.
    let watermarked_img_bytes = img.as_bytes().to_vec();

    // Assert that the image content was modified, proving the watermark was drawn.
    assert_ne!(
        original_img_bytes,
        watermarked_img_bytes,
        "CJK watermark should have been applied, changing the image content"
    );

    Ok(())
}
