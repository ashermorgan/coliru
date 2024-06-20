use std::env::set_current_dir;
use std::path::Path;
use super::manifest::{CopyOptions, Manifest, parse_manifest_file};
use super::tags::tags_match;
use super::utils::copy_file;

/// Execute the steps in a coliru manifest file according to a set of tag rules
pub fn execute_manifest_file(path: &Path, tag_rules: Vec<String>) {
    match parse_manifest_file(path) {
        Ok(manifest) => execute_manifest(manifest, tag_rules),
        Err(why) => eprintln!("Error: {}", why),
    };
}

/// Execute the steps in a coliru manifest according to a set of tag rules
fn execute_manifest(manifest: Manifest, tag_rules: Vec<String>) {
    if let Err(why) = set_current_dir(manifest.base_dir) {
        eprintln!("Error: {}", why);
        return;
    }

    for (i, step) in manifest.steps.iter().enumerate() {
        if tags_match(&tag_rules, &step.tags) {
            println!("Step {}:", i+1);
            execute_copies(&step.copy);
        } else {
            println!("Step {}: skipped", i+1);
        }
    }
}

/// Execute the copy commands specified in a coliru manifest step
fn execute_copies(copies: &[CopyOptions]) {
    for copy in copies {
        println!("  Copy {} to {}", copy.src, copy.dst);
        if let Err(why) = copy_file(&copy.src, &copy.dst) {
            eprintln!("    Error: {}", why);
        }
    }
}
