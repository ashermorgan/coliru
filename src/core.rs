//! Manifest execution functions

use std::env::set_current_dir;
use std::path::Path;
use super::manifest::{CopyLinkOptions, RunOptions, parse_manifest_file};
use super::tags::tags_match;
use super::local::{copy_file, link_file, run_command};
use super::ssh::{resolve_path, send_command, send_staged_files, stage_file};
use tempfile::tempdir;

/// The base directory for SSH installs, relative to the home directory
const SSH_INSTALL_DIR: &str = ".coliru";

/// Performs a dry-run check inside of a loop
///
/// Will print `(DRY RUN)` and then continue to next loop iteration if `dry_run`
/// evaluates to `true`.
macro_rules! check_dry_run {
    ($dry_run:expr) => {
        if $dry_run {
            println!(" (DRY RUN)");
            continue;
        }
        println!("");
    }
}

/// Executes the steps in a coliru manifest file according to a set of tag rules
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

/// Executes a set of copy commands
fn execute_copies(copies: &[CopyLinkOptions], host: &str, staging_dir: &Path,
                  dry_run: bool, step_str: &str) {

    for copy in copies {
        // Resolve relative dst paths if installing over SSH
        let _dst = if host != "" {
            resolve_path(&copy.dst, &format!("~/{}", SSH_INSTALL_DIR))
        } else {
            copy.dst.clone()
        };

        print!("{} Copy {} to ", step_str, copy.src);
        if host != "" {
            print!("{}:", host);
        }
        print!("{}", _dst);

        check_dry_run!(dry_run);

        if host == "" {
            if let Err(why) = copy_file(&copy.src, &_dst) {
                eprintln!("  Error: {}", why);
            }
        } else {
            if let Err(why) = stage_file(&copy.src, &_dst, staging_dir) {
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

/// Executes a set of link commands
fn execute_links(links: &[CopyLinkOptions], dry_run: bool, step_str: &str) {
    for link in links {
        print!("{} Link {} to {}", step_str, link.src, link.dst);

        check_dry_run!(dry_run);

        if let Err(why) = link_file(&link.src, &link.dst) {
            eprintln!("  Error: {}", why);
        }
    }
}

/// Executes a set of run commands
fn execute_runs(runs: &[RunOptions], tag_rules: &[String], host: &str,
                staging_dir: &Path, dry_run: bool, step_str: &str) {

    if host != "" {
        // Copy scripts to remote machine
        let run_copies: Vec<CopyLinkOptions> = runs.iter().map(|x| {
            CopyLinkOptions { src: x.src.clone(), dst: x.src.clone() }
        }).collect();

        execute_copies(&run_copies, host, staging_dir, dry_run, step_str);
    }

    for run in runs {
        let postfix = run.postfix.replace("$COLIRU_RULES",
                                          &tag_rules.join(" "));
        let cmd = format!("{} {} {}", run.prefix, run.src, postfix);

        print!("{} Run {}", step_str, cmd);
        if host != "" {
            print!(" on {}", host);
        }

        check_dry_run!(dry_run);

        if host == "" {
            if let Err(why) = run_command(&cmd) {
                eprintln!("  Error: {}", why);
            }
        } else {
            let ssh_cmd = format!("cd {} && {}", SSH_INSTALL_DIR, &cmd);
            if let Err(why) = send_command(&ssh_cmd, host) {
                eprintln!("  Error: {}", why);
            }
        }
    }
}
