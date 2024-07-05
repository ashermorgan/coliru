//! Manifest execution functions

use std::env::set_current_dir;
use std::path::Path;
use super::manifest::{CopyLinkOptions, RunOptions, parse_manifest_file};
use super::tags::tags_match;
use super::local::{copy_file, link_file, run_command};
use super::ssh::{send_command, send_staged_files, stage_file};
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

        execute_copies(&step.copy, host, temp_dir.path(), dry_run, &step_str);

        if !copy && host == "" {
            execute_links(&step.link, dry_run, &step_str);
        } else {
            execute_copies(&step.link, host, temp_dir.path(), dry_run,
                           &step_str);
        }

        execute_runs(&step.run, &tag_rules, host, temp_dir.path(), dry_run,
                     &step_str);
    }
}

/// Execute a set of copy commands
fn execute_copies(copies: &[CopyLinkOptions], host: &str, staging_dir: &Path,
                  dry_run: bool, step_str: &str) {

    for copy in copies {
        if host == "" {
            print!("{} Copy {} to {}", step_str, copy.src, copy.dst);
        } else {
            print!("{} Copy {} to {}:{}", step_str, copy.src, host, copy.dst);
        }

        if dry_run {
            println!(" (DRY RUN)");
            continue;
        }
        println!("");

        if host == "" {
            if let Err(why) = copy_file(&copy.src, &copy.dst) {
                eprintln!("  Error: {}", why);
            }
        } else {
            if let Err(why) = stage_file(&copy.src, &copy.dst, staging_dir) {
                eprintln!("  Error: {}", why);
            }
        }
    }

    if !dry_run {
        if let Err(why) = send_staged_files(staging_dir, host) {
            eprintln!("  Error: {}", why);
        }
    }
}

/// Execute a set of link commands
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

/// Execute a set of run commands
fn execute_runs(runs: &[RunOptions], tag_rules: &[String], host: &str,
                staging_dir: &Path, dry_run: bool, step_str: &str) {

    if !dry_run && host != "" {
        for run in runs {
            if let Err(why) = stage_file(&run.src, &run.src, staging_dir) {
                eprintln!("Error: {}", why);
            }
        }

        if let Err(why) = send_staged_files(staging_dir, host) {
            eprintln!("Error: {}", why);
        }
    }

    for run in runs {
        let postfix = run.postfix.replace("$COLIRU_RULES", &tag_rules.join(" "));
        let cmd = format!("{} {} {}", run.prefix, run.src, postfix);
        if host == "" {
            print!("{} Run {}", step_str, cmd);
        } else {
            print!("{} Run {} on {}", step_str, cmd, host);
        }

        if dry_run {
            println!(" (DRY RUN)");
            continue;
        }
        println!("");

        if host == "" {
            if let Err(why) = run_command(&cmd) {
                eprintln!("  Error: {}", why);
            }
        } else {
            let ssh_cmd = format!("cd .coliru && {}", &cmd);
            if let Err(why) = send_command(&ssh_cmd, host) {
                eprintln!("  Error: {}", why);
            }
        }
    }
}
