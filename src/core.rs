use std::env::set_current_dir;
use super::manifest;
use super::tags;
use super::utils::copy_file;

/// Execute the steps in a coliru manifest according to a set of tag rules
pub fn execute_manifest(manifest: manifest::Manifest, rules: Vec<String>) {
    if let Err(why) = set_current_dir(manifest.base_dir) {
        eprintln!("Error: {}", why);
        return;
    }

    for (i, step) in manifest.steps.iter().enumerate() {
        if tags::tags_match(&rules, &step.tags) {
            println!("Step {}:", i+1);
            execute_copies(&step.copy);
        } else {
            println!("Step {}: skipped", i+1);
        }
    }
}

/// Execute the copy commands specified in a coliru manifest step
fn execute_copies(copies: &[manifest::CopyOptions]) {
    for copy in copies {
        println!("  Copy {} to {}", copy.src, copy.dst);
        if let Err(why) = copy_file(&copy.src, &copy.dst) {
            eprintln!("    Error: {}", why);
        }
    }
}
