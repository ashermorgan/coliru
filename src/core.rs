use std::env::set_current_dir;
use std::path::Path;
use super::manifest::{CopyLinkOptions, RunOptions, Manifest, parse_manifest_file
};
use super::tags::tags_match;
use super::utils::{copy_file, link_file, run_script};

/// Execute the steps in a coliru manifest file according to a set of tag rules
pub fn execute_manifest_file(path: &Path, tag_rules: Vec<String>, dry_run: bool,
                             copy: bool) {
    match parse_manifest_file(path) {
        Ok(manifest) => execute_manifest(manifest, tag_rules, dry_run, copy),
        Err(why) => eprintln!("Error: {}", why),
    };
}

/// Execute the steps in a coliru manifest according to a set of tag rules
fn execute_manifest(manifest: Manifest, tag_rules: Vec<String>, dry_run: bool,
                    copy: bool) {
    if let Err(why) = set_current_dir(manifest.base_dir) {
        eprintln!("Error: {}", why);
        return;
    }

    for (i, step) in manifest.steps.iter().enumerate() {
        if !tags_match(&tag_rules, &step.tags) { continue; }

        let step_str = format!("[{}/{}]", i+1, manifest.steps.len());

        execute_copies(&step.copy, dry_run, &step_str);
        if copy {
            execute_copies(&step.link, dry_run, &step_str);
        } else {
            execute_links(&step.link, dry_run, &step_str);
        }
        execute_runs(&step.run, &tag_rules, dry_run, &step_str);
    }
}

/// Execute the copy commands specified in a coliru manifest step
fn execute_copies(copies: &[CopyLinkOptions], dry_run: bool, step_str: &str) {
    for copy in copies {
        print!("{} Copy {} to {}", step_str, copy.src, copy.dst);

        if dry_run {
            println!(" (DRY RUN)");
            continue;
        }
        println!("");

        if let Err(why) = copy_file(&copy.src, &copy.dst) {
            eprintln!("  Error: {}", why);
        }
    }
}

/// Execute the link commands specified in a coliru manifest step
fn execute_links(links: &[CopyLinkOptions], dry_run: bool, step_str: &str) {
    for link in links {
        print!("{} Link {} to {}", step_str, link.src, link.dst);

        if dry_run {
            println!(" (DRY RUN)");
            continue;
        }
        println!("");

        if let Err(why) = link_file(&link.src, &link.dst) {
            eprintln!("  Error: {}", why);
        }
    }
}

/// Execute the run commands specified in a coliru manifest step
fn execute_runs(runs: &[RunOptions], tag_rules: &[String], dry_run: bool,
                step_str: &str) {

    for run in runs {
        let postfix = run.postfix.replace("$COLIRU_RULES", &tag_rules.join(" "));
        print!("{} Run {} {} {}", step_str, run.prefix, run.src, postfix);

        if dry_run {
            println!(" (DRY RUN)");
            continue;
        }
        println!("");

        if let Err(why) = run_script(&run.src, &run.prefix, &postfix) {
            eprintln!("  Error: {}", why);
        }
    }
}
