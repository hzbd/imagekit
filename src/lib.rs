pub mod assets;
pub mod cli;
pub mod errors;
pub mod processor;

use anyhow::{Context, Result};
use rayon::prelude::*;
use rusttype::Font;
use std::fs;
use std::path::PathBuf;

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
    let font_data = Asset::get("Roboto-Regular.ttf")
        .context("Could not find font 'Roboto-Regular.ttf' in embedded assets.")?;
    let font = Font::try_from_bytes(font_data.data.as_ref())
        .context("Error constructing Font from embedded data")?;

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

    // 使用 Rayon 并行处理图片 (调用 processor 模块的函数)
    image_paths.par_iter().for_each(|path| {
        if let Err(e) = process_image(path, &cli, &font) {
            eprintln!("Failed to process {}: {}", path.display(), e);
        }
    });

    println!("Image processing complete!");
    Ok(())
}