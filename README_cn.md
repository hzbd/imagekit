# ImageKit

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](https://github.com/hzbd/imagekit)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**ImageKit** 是一个强大且快速的命令行工具，用于批量处理图片。它使用 Rust 编写，通过并行处理来最大化性能，让你能轻松地对整个目录的图片进行尺寸调整和添加水印。

[English](./README.md)

## ✨ 功能特性

- **批量处理**: 递归地处理指定输入目录下的所有图片 (`jpg`, `png`, `gif`, `bmp`)。
- **智能缩放**:
    - 如果只提供宽度，则自动按比例计算高度。
    - 如果只提供高度，则自动按比例计算宽度。
    - 如果同时提供宽高，则精确缩放至指定尺寸（可能会拉伸）。
- **文本水印**: 在图片的九个标准位置添加自定义文本水印。
- **可定制水印**: 自由设置水印的文字内容、字体大小和位置。
- **⚡ 极速性能**: 利用 [Rayon](https://github.com/rayon-rs/rayon) 库并行处理图片，充分利用多核 CPU 的性能。
- **跨平台**: 可在 Windows, macOS, 和 Linux 上编译和运行。
- **零依赖**: 编译后的可执行文件不依赖任何外部库，方便分发。

## ⚙️ 安装与构建

你需要先安装 [Rust 和 Cargo](https://www.rust-lang.org/tools/install)。

1.  **克隆仓库**
    ```bash
    git clone https://github.com/hzbd/imagekit.git
    cd imagekit
    ```

2.  **构建项目**
    为了获得最佳性能，我们构建 release 版本。
    ```bash
    cargo build --release
    ```

3.  **找到可执行文件**
    构建完成后，可执行文件会位于 `target/release/` 目录下。
    -   在 Windows 上是 `target/release/imagekit.exe`
    -   在 macOS / Linux 上是 `target/release/imagekit`

## 🚀 使用方法

### 基本语法

```bash
# 在 Linux / macOS 上
./target/release/imagekit --input-dir <输入目录> --output-dir <输出目录> [选项]

# 在 Windows 上
.\target\release\imagekit.exe --input-dir <输入目录> --output-dir <输出目录> [选项]
```

### 示例

假设你有一个名为 `input_photos` 的文件夹，想把处理后的图片保存到 `processed_photos`。

#### 示例 1: 将所有图片宽度缩放到 800px，高度按比例自动调整
这是最常见的缩放场景，可以保证图片不变形。
```bash
./target/release/imagekit -i ./input_photos -o ./processed_photos --width 800
```

#### 示例 2: 将所有图片高度缩放到 600px，宽度按比例自动调整
```bash
./target/release/imagekit -i ./input_photos -o ./processed_photos --height 600
```

#### 示例 3: 在右下角添加版权水印（不改变尺寸）
```bash
./target/release/imagekit -i ./input_photos -o ./processed_photos --watermark-text "© 2024 My Photos"
```

#### 示例 4: 强制缩放到 1920x1080 并添加居中水印
如果你需要图片有精确的尺寸，即使会拉伸变形。
```bash
./target/release/imagekit \
    -i ./input_photos \
    -o ./processed_photos \
    --width 600 \
    --height 400 \
    --watermark-text "Vacation Memories" \
    --watermark-position center \
    --font-size 96
```

#### 示例 5: 添加一个半透明的红色水印
```bash
./target/release/imagekit \
    -i ./input_photos \
    -o ./processed_photos \
    --watermark-text "DRAFT" \
    --watermark-color FF000080 \
    --font-size 128
```

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