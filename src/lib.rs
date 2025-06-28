pub mod assets;
pub mod cli;
pub mod errors;
pub mod processor;

use anyhow::{Context, Result};
use rayon::prelude::*;
use rusttype::Font;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use assets::Asset;
use cli::Cli;
use processor::process_image;

// 这个 run 函数现在是库的公共 API 的一部分
pub fn run(cli: Cli) -> Result<()> {
    // 检查并创建输出目录
    if !cli.output_dir.exists() {
        fs::create_dir_all(&cli.output_dir)?;
    }

    // 从嵌入的资源加载字体 (从 assets 模块)
    // 1. 加载主字体数据并获得其所有权
    let primary_font_data = Asset::get("Roboto-Regular.ttf")
        .context("Could not find font 'Roboto-Regular.ttf'")?;
    // .into_owned() 将 Cow<'static, [u8]> 转换为 Vec<u8>，我们现在拥有了数据。
    let primary_font_vec: Vec<u8> = primary_font_data.data.into_owned();

    // 2. 加载备用 CJK 字体数据并获得其所有权
    let fallback_font_data = Asset::get("SourceHanSansSC-Regular.otf")
        .context("Could not find CJK font 'SourceHanSansSC-Regular.otf'")?;
    let fallback_font_vec: Vec<u8> = fallback_font_data.data.into_owned();

    // 3. 从我们拥有的数据中创建 Font 对象
    //    由于 Vec<u8> 是 'static 的，所以 Font 也是 Font<'static>
    let primary_font = Font::try_from_vec(primary_font_vec)
        .context("Error constructing primary font")?;
    let fallback_font = Font::try_from_vec(fallback_font_vec)
        .context("Error constructing CJK font")?;

    let fonts = Arc::new(vec![primary_font, fallback_font]);
    
    // 收集所有图片路径
    let image_paths: Vec<PathBuf> = walkdir::WalkDir::new(&cli.input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file() &&
                e.path().extension().and_then(|s| s.to_str()).map_or(false, |s| {
                    matches!(s.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp")
                })
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    if image_paths.is_empty() {
        println!("No images found in the input directory.");
        return Ok(());
    }

    println!("Found {} images to process.", image_paths.len());

    // 使用 Rayon 并行处理图片
    image_paths.par_iter().for_each(move |path| {
        // 克隆 Arc 指针，这是一个轻量级的操作
        let fonts_clone = Arc::clone(&fonts);
        // Rust 会自动将 &Arc<Vec<Font>> 解引用为 &[Font]
        if let Err(e) = process_image(path, &cli, &fonts_clone) {
            eprintln!("Failed to process {}: {}", path.display(), e);
        }
    });

    println!("Image processing complete!");
    Ok(())
}