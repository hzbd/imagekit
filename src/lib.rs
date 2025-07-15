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

// The `run` function is now part of the library's public API.
pub fn run(cli: Cli) -> Result<()> {
    // Check and create the output directory if it doesn't exist.
    if !cli.output_dir.exists() {
        fs::create_dir_all(&cli.output_dir)?;
    }

    let primary_font_data = Asset::get("Roboto-Regular.ttf")
        .context("Could not find font 'Roboto-Regular.ttf'")?;
    let primary_font_vec: Vec<u8> = primary_font_data.data.into_owned();
    let primary_font = Font::try_from_vec(primary_font_vec)
        .context("Error constructing primary font")?;

    let cjk_font_data = Asset::get("SourceHanSansSC-Regular.otf")
        .context("Could not find CJK font 'SourceHanSansSC-Regular.otf'")?;
    let cjk_font_vec: Vec<u8> = cjk_font_data.data.into_owned();
    let cjk_font = Font::try_from_vec(cjk_font_vec)
        .context("Error constructing CJK font")?;

    let thai_font_data = Asset::get("NotoSansThai-Regular.ttf")
        .context("Could not find Thai font 'NotoSansThai-Regular.ttf'")?;
    let thai_font_vec: Vec<u8> = thai_font_data.data.into_owned();
    let thai_font = Font::try_from_vec(thai_font_vec)
        .context("Error constructing Thai font")?;

    let fonts = Arc::new(vec![primary_font, cjk_font, thai_font]);

    // Collect all image paths from the input directory.
    let image_paths: Vec<PathBuf> = walkdir::WalkDir::new(&cli.input_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file() &&
                e.path().extension().and_then(|s| s.to_str()).map_or(false, |s| {
                    matches!(s.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp")
                })
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    if image_paths.is_empty() {
        println!("No images found in the input directory.");
        return Ok(());
    }

    println!("Found {} images to process.", image_paths.len());

    // Use Rayon to process images in parallel.
    image_paths.par_iter().for_each(move |path| {
        // Clone the Arc pointer, which is a lightweight operation.
        let fonts_clone = Arc::clone(&fonts);
        // Rust automatically dereferences `&Arc<Vec<Font>>` to `&[Font]`.
        if let Err(e) = process_image(path, &cli, &fonts_clone) {
            eprintln!("Failed to process {}: {}", path.display(), e);
        }
    });

    println!("Image processing complete!");
    Ok(())
}
