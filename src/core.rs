use std::env::set_current_dir;
use std::path::Path;
use super::manifest::{CopyLinkOptions, RunOptions, parse_manifest_file};
use super::tags::tags_match;
use super::local::{copy_file, link_file, run_script};
use super::ssh::{send_staged_files, stage_file};
use tempfile::tempdir;

/// Execute the steps in a coliru manifest file according to a set of tag rules
pub fn execute_manifest_file(path: &Path, tag_rules: Vec<String>, host: &str,
                             dry_run: bool, copy: bool) {

    let _manifest = parse_manifest_file(path);
    if let Err(why) = _manifest {
        eprintln!("Error: {}", why);
        return;
    }
    let manifest = _manifest.unwrap();

    let _temp_dir = tempdir();
    if let Err(why) = _temp_dir {
        eprintln!("Error: {}", why);
        return;
    }
    let temp_dir = _temp_dir.unwrap();

    if let Err(why) = set_current_dir(manifest.base_dir) {
        eprintln!("Error: {}", why);
        return;
    }

    for (i, step) in manifest.steps.iter().enumerate() {
        if !tags_match(&tag_rules, &step.tags) { continue; }

        let step_str = format!("[{}/{}]", i+1, manifest.steps.len());

        if host == "" {
            execute_copies(&step.copy, dry_run, &step_str);
        } else {
            execute_copies_remote(&step.copy, host, temp_dir.path(), dry_run,
                                  &step_str);
        }
        if !copy && host == "" {
            execute_links(&step.link, dry_run, &step_str);
        } else if host != "" {
            execute_copies_remote(&step.link, host, temp_dir.path(), dry_run,
                                  &step_str);
        } else {
            execute_copies(&step.link, dry_run, &step_str);
        }
        execute_runs(&step.run, &tag_rules, host, dry_run, &step_str);
    }
}

/// Execute a set of copy commands on the local machine
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

/// Execute a set of copy commands on a remote machine
fn execute_copies_remote(copies: &[CopyLinkOptions], host: &str,
                         staging_dir: &Path, dry_run: bool, step_str: &str) {

    for copy in copies {
        print!("{} Copy {} to {}:{}", step_str, copy.src, host, copy.dst);

        if dry_run {
            println!(" (DRY RUN)");
            continue;
        }
        println!("");

        if let Err(why) = stage_file(&copy.src, &copy.dst, staging_dir) {
            eprintln!("  Error: {}", why);
        }
    }

    if !dry_run {
        if let Err(why) = send_staged_files(staging_dir, host) {
            eprintln!("  Error: {}", why);
        }
    }
}

/// Execute a set of link commands on the local machine
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

/// Execute a set of run commands on the local machine
fn execute_runs(runs: &[RunOptions], tag_rules: &[String], host: &str,
                dry_run: bool, step_str: &str) {

    for run in runs {
        let postfix = run.postfix.replace("$COLIRU_RULES", &tag_rules.join(" "));
        if host == "" {
            print!("{} Run {} {} {}", step_str, run.prefix, run.src, postfix);
        } else {
            print!("{} Run {} {} {} on {}", step_str, run.prefix, run.src,
                   postfix, host);
        }

        if dry_run {
            println!(" (DRY RUN)");
            continue;
        }
        println!("");

        if host == "" {
            if let Err(why) = run_script(&run.src, &run.prefix, &postfix) {
                eprintln!("  Error: {}", why);
            }
        } else {
            eprintln!("  Error: not implemented");
        }
    }
}
