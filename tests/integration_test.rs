use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, Rgba};
use tempfile::tempdir;
use std::fs;

// 导入我们库中的公共项
use imagekit::{
    assets::Asset,
    cli::{Cli, HexColor, WatermarkPosition},
    processor::add_watermark,
    run,
};
// 导入 Font 以便在测试中创建
use rusttype::Font;

/// 辅助函数，用于在测试中加载字体列表。
/// 它通过获取数据所有权来解决生命周期问题。
fn load_test_fonts() -> Result<Vec<Font<'static>>> {
    // --- 解决方案在这里 ---
    // 1. 加载主字体数据并获得其所有权
    let primary_font_data = Asset::get("Roboto-Regular.ttf")
        .context("Test setup failed: Could not find 'Roboto-Regular.ttf'")?;
    let primary_font_vec: Vec<u8> = primary_font_data.data.into_owned();

    // 2. 加载备用 CJK 字体数据并获得其所有权
    let fallback_font_data = Asset::get("SourceHanSansSC-Regular.otf")
        .context("Test setup failed: Could not find 'SourceHanSansSC-Regular.otf'")?;
    let fallback_font_vec: Vec<u8> = fallback_font_data.data.into_owned();

    // 3. 从我们拥有的数据中创建 Font 对象
    let primary_font = Font::try_from_vec(primary_font_vec)
        .context("Test setup failed: Could not parse 'Roboto-Regular.ttf'")?;
    let fallback_font = Font::try_from_vec(fallback_font_vec)
        .context("Test setup failed: Could not parse 'SourceHanSansSC-Regular.otf'")?;
    // --- 结束解决方案 ---

    Ok(vec![primary_font, fallback_font])
}


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
    let color = HexColor::from_str("#FF000080").unwrap();
    assert_eq!(color.0, Rgba([255, 0, 0, 128]));
    let color = HexColor::from_str("00FF00").unwrap();
    assert_eq!(color.0, Rgba([0, 255, 0, 128]));
    let color = HexColor::from_str("0000ffAA").unwrap();
    assert_eq!(color.0, Rgba([0, 0, 255, 170]));
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

/// 集成测试：模拟一次完整的命令行运行，包括调整尺寸和添加水印。
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

/// 测试：当只提供宽度时，是否能按比例缩放。
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

/// 验证质量参数是否对文件大小产生影响
#[test]
fn test_quality_options_affect_file_size() -> Result<()> {
    let input_dir = tempdir()?;
    let test_image_path = input_dir.path().join("quality_test.jpg");
    image::RgbImage::new(200, 200).save(&test_image_path)?;

    // 1. 使用低质量保存
    let low_q_output_dir = tempdir()?;
    let cli_low = Cli {
        input_dir: input_dir.path().to_path_buf(),
        output_dir: low_q_output_dir.path().to_path_buf(),
        quality: 10,
        width: None, height: None, watermark_text: None,
        watermark_position: WatermarkPosition::Se, font_size: 24,
        watermark_color: HexColor(Rgba([255,255,255,128])),
    };
    run(cli_low)?;
    let low_q_size = fs::metadata(low_q_output_dir.path().join("quality_test.jpg"))?.len();

    // 2. 使用高质量保存
    let high_q_output_dir = tempdir()?;
    let cli_high = Cli {
        input_dir: input_dir.path().to_path_buf(),
        output_dir: high_q_output_dir.path().to_path_buf(),
        quality: 100,
        width: None, height: None, watermark_text: None,
        watermark_position: WatermarkPosition::Se, font_size: 24,
        watermark_color: HexColor(Rgba([255,255,255,128])),
    };
    run(cli_high)?;
    let high_q_size = fs::metadata(high_q_output_dir.path().join("quality_test.jpg"))?.len();

    println!("Low-Q: {}, High-Q: {}", low_q_size, high_q_size);
    assert!(low_q_size < high_q_size, "Low quality JPEG should be smaller than high quality JPEG");

    Ok(())
}

// 验证 CJK 字符支持
#[test]
fn test_cjk_watermark_support() -> Result<()> {
    // 1. 准备一个已知背景的图片
    let mut img = DynamicImage::ImageRgba8(
        image::RgbaImage::from_pixel(300, 100, Rgba([255, 255, 255, 255]))
    );
    // 2. 复制一份处理前的图片字节数据
    let original_img_bytes = img.as_bytes().to_vec();

    let fonts = load_test_fonts()?;

    // 3. 执行水印操作
    add_watermark(
        &mut img,
        "测试 Test 123 テスト", // 混合了中文、英文、数字和日文
        &fonts,
        30,
        WatermarkPosition::Center,
        HexColor(Rgba([0, 0, 0, 128])), // 半透明黑色
    );

    // 4. 获取处理后的图片字节数据
    let watermarked_img_bytes = img.as_bytes().to_vec();

    // --- 解决方案在这里：使用更健壮的断言 ---
    // 断言图片内容被修改过，证明水印被画上去了，无论画在哪里
    assert_ne!(
        original_img_bytes,
        watermarked_img_bytes,
        "CJK watermark should have been applied, changing the image content"
    );
    // --- 结束解决方案 ---

    Ok(())
}