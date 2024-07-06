//! Manifest execution functions

use std::env::set_current_dir;
use std::fmt::Display;
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

/// Handles minor errors that occur during command execution and returns a bool
/// indicating whether an error occurred
fn handle_error<T: Display>(result: Result<(), T>) -> bool {
    if let Err(why) = result {
        eprintln!("  Error: {}", why);
        return true;
    }
    false
}

/// Executes the steps in a coliru manifest file according to a set of tag rules
///
/// Returns an Err if a critical err occurs and returns a bool indicating
/// whether any minor errors occurred otherwise
pub fn execute_manifest_file(path: &Path, tag_rules: Vec<String>, host: &str,
                             dry_run: bool, copy: bool) -> Result<bool, String> {

    let manifest = parse_manifest_file(path).map_err(|why| why.to_string())?;
    let temp_dir = tempdir().map_err(|why| why.to_string())?;
    set_current_dir(manifest.base_dir).map_err(|why| why.to_string())?;

    let mut errors = false;

    for (i, step) in manifest.steps.iter().enumerate() {
        if !tags_match(&tag_rules, &step.tags) { continue; }

        let step_str = format!("[{}/{}]", i+1, manifest.steps.len());

        errors |= execute_copies(&step.copy, host, temp_dir.path(), dry_run,
                                 &step_str);

        if !copy && host == "" {
            errors |= execute_links(&step.link, dry_run, &step_str);
        } else {
            errors |= execute_copies(&step.link, host, temp_dir.path(), dry_run,
                           &step_str);
        }

        errors |= execute_runs(&step.run, &tag_rules, host, temp_dir.path(),
                               dry_run, &step_str);
    }

    Ok(errors)
}

/// Executes a set of copy commands and returns a bool indicating whether any
/// error occurred
fn execute_copies(copies: &[CopyLinkOptions], host: &str, staging_dir: &Path,
                  dry_run: bool, step_str: &str) -> bool {

    let mut errors = false;

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
            errors |= handle_error(copy_file(&copy.src, &_dst));
        } else {
            errors |= handle_error(stage_file(&copy.src, &_dst, staging_dir));
        }
    }

    if !dry_run {
        errors |= handle_error(send_staged_files(staging_dir, host));
    }

    errors
}

/// Executes a set of link commands and returns a bool indicating whether any
/// error occurred
fn execute_links(links: &[CopyLinkOptions], dry_run: bool, step_str: &str)
    -> bool {

    let mut errors = false;

    for link in links {
        print!("{} Link {} to {}", step_str, link.src, link.dst);

        check_dry_run!(dry_run);

        errors |= handle_error(link_file(&link.src, &link.dst));
    }

    errors
}

/// Executes a set of run commands and returns a bool indicating whether any
/// error occurred
fn execute_runs(runs: &[RunOptions], tag_rules: &[String], host: &str,
                staging_dir: &Path, dry_run: bool, step_str: &str) -> bool {

    let mut errors = false;

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
            errors |= handle_error(run_command(&cmd));
        } else {
            let ssh_cmd = format!("cd {} && {}", SSH_INSTALL_DIR, &cmd);
            errors |= handle_error(send_command(&ssh_cmd, host));
        }
    }

    errors
}
