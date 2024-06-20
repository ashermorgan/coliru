use clap::{Parser, ColorChoice};
use std::path::Path;
use super::core::execute_manifest_file;

/// Stores arguments to the coliru CLI
#[derive(Parser, Debug)]
#[command(version, color=ColorChoice::Never,
          about="A minimal, flexible, dotfile installer")]
struct Args {
    /// The path to the coliru YAML manifest file
    pub manifest: String,

    /// The set of tag rules to enforce
    #[arg(short, long, num_args=0..)]
    pub tag_rules: Vec<String>,

    /// Copy files instead of creating links
    #[arg(short, long)]
    pub copy: bool,

    /// Do a trial run without any permanent changes
    #[arg(short = 'n', long)]
    pub dry_run: bool,
}

/// Runs the coliru CLI
pub fn run() {
    let args = Args::parse();
    let manifest_path = Path::new(&args.manifest);
    execute_manifest_file(&manifest_path, args.tag_rules, args.dry_run,
                          args.copy);
}
