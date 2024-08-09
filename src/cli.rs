//! The coliru command line interface

use anyhow::{Context, Result};
use colored::{Colorize, control::set_override};
use clap::{Parser, ColorChoice};
use std::path::Path;
use super::core::{install_manifest, list_tags};
use super::manifest::parse_manifest_file;

/// CLI about description
const HELP_ABOUT: &str = "A minimal, flexible, dotfile installer";

/// CLI examples to be appended to help output
const HELP_EXAMPLES: &str = "\
Examples:
  # List tags in manifest
  coliru manifest.yml --list-tags

  # Preview installation steps with tags matching A && (B || C) && !D
  coliru manifest.yml --tag-rules A B,C ^D --dry-run

  # Install dotfiles on local machine
  coliru manifest.yml --tag-rules A B,C ^D

  # Install dotfiles to user@hostname over SSH
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

    /// List available tags and quit without installing
    #[arg(short, long)]
    pub list_tags: bool,

    /// Do a trial run without any permanent changes
    #[arg(short = 'n', long)]
    pub dry_run: bool,

    /// Install dotfiles on another machine over SSH
    #[arg(long, default_value="", hide_default_value=true)]
    pub host: String,

    /// Interpret link commands as copy commands
    #[arg(long)]
    pub copy: bool,

    /// Disable color output
    #[arg(long)]
    pub no_color: bool,
}

/// Runs the coliru CLI
pub fn run() {
    let args = Args::parse();

    match run_args(args) {
        Err(why) => {
            eprintln!("{} {:#}", "Error:".bold().red(), why);
            std::process::exit(2);
        },
        Ok(minor_errors) => {
            std::process::exit(if minor_errors { 1 } else { 0 });
        },
    }
}

/// Runs the coliru CLI according to a set of arguments
///
/// Returns an Err if a critical occurs, Ok(true) if minor errors occurred, and
/// Ok(false) if no errors occurred.
fn run_args(args: Args) -> Result<bool> {
    if args.no_color {
        set_override(false);
    }

    let manifest = parse_manifest_file(Path::new(&args.manifest))
        .with_context(|| {
            format!("Failed to parse {}", args.manifest)
        })?;

    if args.list_tags {
        list_tags(manifest);
        Ok(false)
    } else {
        install_manifest(manifest, args.tag_rules, &args.host, args.dry_run,
                         args.copy)
    }
}
