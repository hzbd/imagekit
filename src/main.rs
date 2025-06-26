use anyhow::Result;
use clap::Parser;
use imagekit::cli::Cli; // 从我们自己的库中导入 Cli

fn main() -> Result<()> {
    // 1. 解析命令行参数
    let cli = Cli::parse();
    // 2. 调用库中的核心运行逻辑
    imagekit::run(cli)
}