//! The coliru command line interface

use clap::{Parser, ColorChoice};
use std::path::Path;
use super::core::execute_manifest_file;

/// CLI about description
const HELP_ABOUT: &str = "A minimal, flexible, dotfile installer";

/// CLI examples to be appended to help output
const HELP_EXAMPLES: &str = "\
Examples:
  # Install dotfiles from manifest.yml with tags matching A && (B || C) && !D
  coliru manifest.yml --tag-rules A B,C ^D

  # Install dotfiles from manifest.yml to user@hostname over SSH
  coliru manifest.yml --tag-rules A B,C ^D --host user@hostname";

/// Arguments to the coliru CLI
#[derive(Parser, Debug)]
#[command(version, color=ColorChoice::Never, arg_required_else_help=true,
          about=HELP_ABOUT, after_help=HELP_EXAMPLES)]
struct Args {
    /// The path to the coliru manifest file
    pub manifest: String,

    /// The set of tag rules to enforce
    #[arg(short, long, value_name="RULE", num_args=0..)]
    pub tag_rules: Vec<String>,

    /// Do a trial run without any permanent changes
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Install dotfiles on another machine over SSH
    #[arg(long, default_value="", hide_default_value=true)]
    pub host: String,

    /// Interpret link commands as copy commands
    #[arg(long)]
    pub copy: bool,
}

/// Runs the coliru CLI
pub fn run() {
    let args = Args::parse();
    let manifest_path = Path::new(&args.manifest);

    match execute_manifest_file(&manifest_path, args.tag_rules, &args.host,
                                args.dry_run, args.copy) {
        Err(why) => {
            eprintln!("Error: {}", why);
            std::process::exit(2);
        },
        Ok(minor_errors) => {
            std::process::exit(if minor_errors { 1 } else { 0 });
        },
    }
}
