mod cli;
mod core;
mod manifest;
mod tags;
mod utils;

use clap::Parser;

fn main() {
    let args = cli::CLI::parse();
    match manifest::parse_manifest_file(&std::path::Path::new(&args.manifest)) {
        Ok(manifest) => core::execute_manifest(manifest, args.tag_rules),
        Err(why) => eprintln!("Error: {}", why),
    };
}
