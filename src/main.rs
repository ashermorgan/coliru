mod cli;
mod core;
mod manifest;
mod tags;
mod utils;

use clap::Parser;

fn main() {
    let args = cli::CLI::parse();
    let manifest_path = std::path::Path::new(&args.manifest);
    core::execute_manifest_file(&manifest_path, args.tag_rules);
}
