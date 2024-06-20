use clap::{Parser, ColorChoice};

/// A minimal, flexible, dotfile installer
#[derive(Parser, Debug)]
#[command(version, color=ColorChoice::Never)]
pub struct CLI {
    /// The path to the coliru YAML manifest file
    pub manifest: String,

    /// The set of tag rules to enforce
    #[arg(short, long, num_args=0..)]
    pub tag_rules: Vec<String>,

    /// Do a trial run without any permanent changes
    #[arg(short = 'n', long)]
    pub dry_run: bool,
}
