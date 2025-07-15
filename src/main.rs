use anyhow::Result;
use clap::Parser;
use imagekit::cli::Cli;

fn main() -> Result<()> {
    // 1. Parse command-line arguments.
    let cli = Cli::parse();
    // 2. Call the core run logic from the library.
    imagekit::run(cli)
}
