# ImageKit

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/hzbd/imagekit)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rust-1.87%2B-blue.svg)](https://www.rust-lang.org)

**ImageKit** 是一个强大、快速且灵活的命令行工具，用于批量处理图片。它使用 Rust 编写，通过并行处理来最大化性能，让你能轻松地对整个目录的图片进行尺寸调整、质量控制和添加高度可定制的水印。

## 🌟 功能特性

- **批量处理**: 递归地处理指定输入目录下的所有图片 (`.jpg`, `.jpeg`, `.png`, `.gif`, `.bmp`)。
- **智能缩放**:
    - 如果只提供宽度，则自动按比例计算高度，保证图片不变形。
    - 如果只提供高度，则自动按比例计算宽度。
- **质量控制**: 使用 `-q` 或 `--quality` 参数（1-100）微调输出质量，在文件大小和视觉保真度之间取得平衡。设置为 `100` 可获得最佳质量。
- **高度可定制的水印**:
    - 在图片的九个标准位置添加自定义文本水印。
    - **自定义颜色**: 通过十六进制色码（如 `RRGGBB` 或 `RRGGBBAA`）精确控制水印颜色和透明度。
- **智能水印缩放**: 如果请求的水印对于图片来说过大，工具会自动缩小水印以确保其完整显示，永不裁切。
- **⚡ 极速性能**: 利用 [Rayon](https://github.com/rayon-rs/rayon) 库并行处理图片，充分利用多核 CPU 的性能。
- **跨平台**: 可在 Windows, macOS, 和 Linux 上编译和运行。

## ⚙️ 安装与构建

你需要先安装 [Rust 和 Cargo](https://www.rust-lang.org/tools/install)。

1.  **克隆仓库**
    ```bash
    git clone https://github.com/hzbd/imagekit.git
    cd imagekit
    ```

2.  **构建项目**
    ```bash
    cargo build --release
    ```

3.  **找到可执行文件**
    构建完成后，可执行文件位于 `target/release/` 目录下。

## 🚀 使用方法

### 示例

#### 示例 1: 缩放图片并以最高质量保存
如果你希望调整尺寸但不损失图片质量，请使用 `--quality 100`。
```bash
./target/release/imagekit -i ./input_photos -o ./processed_photos --width 1024 --quality 100
```

#### 示例 2: 缩放图片并指定一个中等质量等级
在文件大小和质量之间取得一个很好的平衡。
```bash
./target/release/imagekit -i ./input_photos -o ./processed_photos --width 1024 -q 75
```

#### 示例 3: 添加一个不透明的黑色水印（使用默认质量 85）
```bash
./target/release/imagekit \
    -i ./input_photos \
    -o ./processed_photos \
    --watermark-text "Confidential" \
    --watermark-color 000000FF
```
> **提示**: 如果只提供6位颜色码（如 `000000`），alpha 通道会默认为半透明 (`80`)。

## 📋 命令行选项

| 选项                 | 标志                 | 描述                                                                    | 必需/可选 | 默认值   |
| -------------------- | -------------------- | ----------------------------------------------------------------------- | --------- | -------- |
| 输入目录             | `-i`, `--input-dir`  | 包含需要处理的图片的源目录。                                            | **必需**  | -        |
| 输出目录             | `-o`, `--output-dir` | 用于存放处理后图片的目录。                                              | **必需**  | -        |
| 宽度                 | `--width`            | （可选）调整图片的宽度。若不提供高度，则按比例缩放。                    | 可选      | 原始宽度 |
| 高度                 | `--height`           | （可选）调整图片的高度。若不提供宽度，则按比例缩放。                    | 可选      | 原始高度 |
| 水印文字             | `--watermark-text`   | （可选）要添加的水印文字内容。                                          | 可选      | -        |
| 水印位置             | `--watermark-position` | （可选）水印在图片上的位置。                                            | 可选      | `se`     |
| 字体大小             | `--font-size`        | （可选）水印文字的大小（单位：像素）。                                  | 可选      | `24`     |
| 水印颜色             | `--watermark-color`  | （可选）水印颜色，格式为 RRGGBB 或 RRGGBBAA。                           | 可选      | `FFFFFF80` (半透明白) |
| 质量                 | `-q`, `--quality`    | （可选）设置输出质量(1-100)。对于JPEG，影响压缩率；对于PNG，影响压缩速度。 | 可选      | `85`     |

#### `watermark-position` 的可用值:

-   `nw`: 左上 (North-West)
-   `north`: 中上
-   `ne`: 右上 (North-East)
-   `west`: 中左
-   `center`: 居中
-   `east`: 中右
-   `sw`: 左下 (South-West)
-   `south`: 中下
-   `se`: 右下 (South-East)

## 🛠️ 开发与测试

如果你想为此项目贡献代码，可以按以下步骤操作：

1.  克隆仓库。
2.  做出你的修改。
3.  运行测试以确保所有功能正常：
    ```bash
    cargo test
    ```

## 📜 许可证

本项目使用 [MIT 许可证](LICENSE)。