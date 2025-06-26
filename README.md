# ImageKit

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/hzbd/imagekit)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.87%2B-blue.svg)](https://www.rust-lang.org)

**ImageKit** is a powerful, fast, and flexible command-line tool for batch image processing. Written in Rust, it leverages parallel processing to maximize performance, allowing you to effortlessly resize, control the quality of, and add highly customizable watermarks to entire directories of images.

## 🌟 Features

- **Batch Processing**: Recursively processes all images (`.jpg`, `.jpeg`, `.png`, `.gif`, `.bmp`) in a specified input directory.
- **Smart Scaling**:
    - If only a width is provided, the height is calculated automatically to maintain the aspect ratio.
    - If only a height is provided, the width is calculated automatically.
- **Quality Control**: Use the `-q` or `--quality` flag (1-100) to fine-tune the output quality, balancing file size and visual fidelity. Set to `100` for the best possible quality.
- **Highly Customizable Watermarks**:
    - Add text watermarks in nine standard positions.
    - **Custom Colors**: Precisely control watermark color and opacity using hex codes (e.g., `RRGGBB` or `RRGGBBAA`).
- **Intelligent Watermark Scaling**: If the requested watermark is too large for an image, it is automatically scaled down to fit perfectly, ensuring it is never cropped.
- **⚡ Blazing Fast Performance**: Utilizes the [Rayon](https://github.com/rayon-rs/rayon) library to process images in parallel, taking full advantage of multi-core CPUs.
- **Cross-Platform**: Compiles and runs on Windows, macOS, and Linux.

## ⚙️ Installation & Build

You will need to have [Rust and Cargo](https://www.rust-lang.org/tools/install) installed.

1.  **Clone the Repository**
    ```bash
    git clone https://github.com/hzbd/imagekit.git
    cd imagekit
    ```

2.  **Build the Project**
    ```bash
    cargo build --release
    ```

3.  **Locate the Executable**
    After building, the executable will be located in the `target/release/` directory.

## 🚀 Usage

### Examples

#### Example 1: Resize images and save at maximum quality
If you want to resize without losing quality, use `--quality 100`.
```bash
./target/release/imagekit -i ./input_photos -o ./processed_photos --width 1024 --quality 100
```

#### Example 2: Resize images and set a medium quality level
A good balance between file size and quality.
```bash
./target/release/imagekit -i ./input_photos -o ./processed_photos --width 1024 -q 75
```

#### Example 3: Add an opaque black watermark (using default quality 85)
```bash
./target/release/imagekit \
    -i ./input_photos \
    -o ./processed_photos \
    --watermark-text "Confidential" \
    --watermark-color 000000FF
```
> **Tip**: If you provide only a 6-digit hex code (e.g., `000000`), the alpha channel will default to semi-transparent (`80`).

## 📋 Command-Line Options

| Option             | Flags                      | Description                                                               | Required/Optional | Default             |
| ------------------ | -------------------------- | ------------------------------------------------------------------------- | ----------------- | ------------------- |
| Input Directory    | `-i`, `--input-dir`        | The source directory containing images to process.                        | **Required**      | -                   |
| Output Directory   | `-o`, `--output-dir`       | The directory where processed images will be saved.                       | **Required**      | -                   |
| Width              | `--width`                  | (Optional) Resize image width. Scales proportionally if height is omitted. | Optional          | Original width      |
| Height             | `--height`                 | (Optional) Resize image height. Scales proportionally if width is omitted. | Optional          | Original height     |
| Watermark Text     | `--watermark-text`         | (Optional) The text content for the watermark.                            | Optional          | -                   |
| Watermark Position | `--watermark-position`     | (Optional) The position of the watermark on the image.                    | Optional          | `se`                |
| Font Size          | `--font-size`              | (Optional) The font size of the watermark text in pixels.                 | Optional          | `24`                |
| Watermark Color    | `--watermark-color`        | (Optional) Watermark color in RRGGBB or RRGGBBAA hex format.              | Optional          | `FFFFFF80` (semi-transparent white) |
| Quality            | `-q`, `--quality`          | (Optional) Set output quality (1-100). Affects JPEG and PNG compression.  | Optional          | `85`                |

#### Available values for `watermark-position`:

-   `nw`: North-West (top-left)
-   `north`: North (top-center)
-   `ne`: North-East (top-right)
-   `west`: West (middle-left)
-   `center`: Center (middle-center)
-   `east`: East (middle-right)
-   `sw`: South-West (bottom-left)
-   `south`: South (bottom-center)
-   `se`: South-East (bottom-right)

## 🛠️ Development & Testing

If you'd like to contribute to the project:

1.  Clone the repository.
2.  Make your changes.
3.  Run tests to ensure all functionality is working as expected:
    ```bash
    cargo test
    ```

## 📜 License

This project is licensed under the [MIT License](LICENSE).
