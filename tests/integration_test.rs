use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, Rgba};
use tempfile::tempdir;

// 导入我们库中的公共项
use imagekit::{
    assets::Asset,
    cli::{Cli, WatermarkPosition},
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

/// 单元测试：验证 `add_watermark` 函数是否确实修改了图片。
///
/// 这个测试不关心水印画在哪里，只关心调用函数后，
/// 图片的整体内容与原始版本不同。
#[test]
fn test_add_watermark_logic() -> Result<()> {
    // 1. 准备：创建一个不透明的黑色背景图片
    let mut img = DynamicImage::ImageRgba8(
        image::RgbaImage::from_pixel(200, 200, Rgba([0, 0, 0, 255]))
    );

    // 克隆一份原始图片数据，用于事后比较
    let original_img_bytes = img.to_bytes();

    // 加载测试所需的字体
    let font_data = Asset::get("Roboto-Regular.ttf").expect("Font not found for testing");
    let font = rusttype::Font::try_from_bytes(font_data.data.as_ref())
        .context("Failed to parse font from embedded data in test")?;

    // 2. 执行：调用我们要测试的函数
    add_watermark(&mut img, "Test", &font, 20, WatermarkPosition::Se);

    // 3. 断言：验证图片内容已经发生了变化
    let watermarked_img_bytes = img.to_bytes();
    assert_ne!(
        original_img_bytes,
        watermarked_img_bytes,
        "Image content should have changed after applying watermark"
    );

    Ok(())
}

/// 集成测试：模拟一次完整的命令行运行，包括调整尺寸和添加水印。
///
/// 这个测试验证了从输入到输出的整个流程是否按预期工作。
#[test]
fn test_full_run_with_resize_and_watermark() -> Result<()> {
    // 1. 准备：创建临时的输入和输出目录
    let input_dir = tempdir()?;
    let output_dir = tempdir()?;

    // 在输入目录中创建一个测试图片
    let test_image_path = input_dir.path().join("test.png");
    let initial_image = image::RgbaImage::from_pixel(200, 200, Rgba([0, 0, 0, 0]));
    initial_image.save(&test_image_path)?;

    // 获取原始图片的字节数据，用于比较
    let original_bytes = std::fs::read(&test_image_path)?;

    // 模拟命令行参数
    let cli = Cli {
        input_dir: input_dir.path().to_path_buf(),
        output_dir: output_dir.path().to_path_buf(),
        width: Some(100),
        height: Some(80),
        watermark_text: Some("Integration Test".to_string()),
        watermark_position: WatermarkPosition::Center,
        font_size: 16,
    };

    // 2. 执行：调用库的主函数
    run(cli)?;

    // 3. 断言：验证结果
    let output_image_path = output_dir.path().join("test.png");
    assert!(output_image_path.exists(), "Output image was not created");

    // 读取处理后图片的字节数据
    let processed_bytes = std::fs::read(&output_image_path)?;

    // 断言图片内容被修改过（因为调整了尺寸和加了水印）
    assert_ne!(original_bytes, processed_bytes, "Image content did not change after processing");

    // 额外断言，确保尺寸被正确修改
    let output_img = image::open(&output_image_path)?;
    assert_eq!(output_img.dimensions(), (100, 80), "Image was not resized correctly");

    Ok(())
}