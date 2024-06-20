use std::env::set_current_dir;
use std::path::Path;
use super::manifest::{CopyOptions, Manifest, parse_manifest_file};
use super::tags::tags_match;
use super::utils::copy_file;

/// Execute the steps in a coliru manifest file according to a set of tag rules
pub fn execute_manifest_file(path: &Path, tag_rules: Vec<String>, dry_run: bool)
{
    match parse_manifest_file(path) {
        Ok(manifest) => execute_manifest(manifest, tag_rules, dry_run),
        Err(why) => eprintln!("Error: {}", why),
    };
}

/// Execute the steps in a coliru manifest according to a set of tag rules
fn execute_manifest(manifest: Manifest, tag_rules: Vec<String>, dry_run: bool) {
    if let Err(why) = set_current_dir(manifest.base_dir) {
        eprintln!("Error: {}", why);
        return;
    }

    for (i, step) in manifest.steps.iter().enumerate() {
        print!("Step {}:", i+1);
        if !tags_match(&tag_rules, &step.tags) {
            println!(" (skipped due to tag rules)");
            continue;
        }
        println!("");

        execute_copies(&step.copy, dry_run);
    }
}

/// Execute the copy commands specified in a coliru manifest step
fn execute_copies(copies: &[CopyOptions], dry_run: bool) {
    for copy in copies {
        print!("  Copy {} to {}", copy.src, copy.dst);
        if dry_run {
            println!(" (skipped due to --dry-run)");
            return;
        }
        println!("");

        if let Err(why) = copy_file(&copy.src, &copy.dst) {
            eprintln!("    Error: {}", why);
        }
    }
}
