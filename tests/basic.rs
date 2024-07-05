//! End to end tests that test general, non-installation, CLI behavior

mod test_utils;

use test_utils::*;
use std::env::consts::EXE_SUFFIX;

#[test]
fn test_basic_help() {
    let (_dirs, mut cmd) = setup_e2e_local("test_basic_help");
    cmd.arg("--help");
    let expected = format!("\
A minimal, flexible, dotfile installer

Usage: coliru{EXE_SUFFIX} [OPTIONS] <MANIFEST>

Arguments:
  <MANIFEST>  The path to the coliru manifest file

Options:
  -t, --tag-rules [<RULE>...]  The set of tag rules to enforce
  -n, --dry-run                Do a trial run without any permanent changes
      --host <HOST>            Install dotfiles on another machine over SSH
      --copy                   Interpret link commands as copy commands
  -h, --help                   Print help
  -V, --version                Print version

Examples:
  # Install dotfiles from manifest.yml with tags matching A && (B || C) && !D
  coliru manifest.yml --tag-rules A B,C ^D

  # Install dotfiles from manifest.yml to user@hostname over SSH
  coliru manifest.yml --tag-rules A B,C ^D --host user@hostname
");
    assert_eq!(stdout_to_string(&mut cmd), expected);
    assert_eq!(&stderr_to_string(&mut cmd), "");
}

#[test]
fn test_basic_empty_manifest() {
    let (dirs, mut cmd) = setup_e2e_local("test_basic_empty_manifest");
    cmd.args(["manifest.yml"]);
    write_file(&dirs.local.join("manifest.yml"), "");

    let expected = "Error: missing field `steps`\n";
    assert_eq!(&stderr_to_string(&mut cmd), expected);
    assert_eq!(&stdout_to_string(&mut cmd), "");
}

#[test]
#[cfg(target_family = "unix")]
fn test_basic_missing_manifest() {
    let (_dirs, mut cmd) = setup_e2e_local("test_basic_missing_manifest");
    cmd.args(["missing.yml"]);

    let expected = "Error: No such file or directory (os error 2)\n";
    assert_eq!(&stderr_to_string(&mut cmd), expected);
    assert_eq!(&stdout_to_string(&mut cmd), "");
}

#[test]
#[cfg(target_family = "windows")]
fn test_basic_missing_manifest() {
    let (_dirs, mut cmd) = setup_e2e_local("test_basic_missing_manifest");
    cmd.args(["missing.yml"]);

    let expected = "Error: The system cannot find the file specified. \
                    (os error 2)\n";
    assert_eq!(&stderr_to_string(&mut cmd), expected);
    assert_eq!(&stdout_to_string(&mut cmd), "");
}

#[test]
#[cfg(target_family = "unix")]
fn test_basic_absolute_manifest() {
    let (dirs, mut cmd) = setup_e2e_local("test_basic_absolute_manifest");
    let manifest_path = dirs.local.join("manifest.yml");
    cmd.args([&manifest_path.to_str().unwrap(), "--dry-run", "-t", "linux"]);

    let expected = "\
[1/3] Copy gitconfig to ~/.gitconfig (DRY RUN)
[2/3] Link bashrc to ~/.bashrc (DRY RUN)
[2/3] Link vimrc to ~/.vimrc (DRY RUN)
[2/3] Run sh script.sh arg1 linux (DRY RUN)
";
    assert_eq!(&stderr_to_string(&mut cmd), "");
    assert_eq!(&stdout_to_string(&mut cmd), expected);

    // Assert files are correctly copied/linked/run
    let bash_exists = dirs.home.join(".bashrc").exists();
    let git_exists = dirs.home.join(".gitconfig").exists();
    let vim1_exists = dirs.home.join(".vimrc").exists();
    let vim2_exists = dirs.home.join("_vimrc").exists();
    let log_exists = dirs.home.join("log.txt").exists();
    assert_eq!(bash_exists, false);
    assert_eq!(git_exists, false);
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_exists, false);
    assert_eq!(log_exists, false);
}

#[test]
#[cfg(target_family = "windows")]
fn test_basic_absolute_manifest() {
    let (dirs, mut cmd) = setup_e2e_local("test_basic_absolute_manifest");
    let manifest_path = dirs.local.join("manifest.yml");
    cmd.args([&manifest_path.to_str().unwrap(), "--dry-run", "-t", "linux"]);

    let expected = "\
[1/3] Copy gitconfig to .gitconfig (DRY RUN)
[2/3] Link bashrc to .bashrc (DRY RUN)
[2/3] Link vimrc to .vimrc (DRY RUN)
[2/3] Run sh script.sh arg1 linux (DRY RUN)
";
    assert_eq!(&stderr_to_string(&mut cmd), "");
    assert_eq!(&stdout_to_string(&mut cmd), expected);

    // Assert files are correctly copied/linked/run
    let bash_exists = dirs.local.join(".bashrc").exists();
    let git_exists = dirs.local.join(".gitconfig").exists();
    let vim1_exists = dirs.local.join(".vimrc").exists();
    let vim2_exists = dirs.local.join("_vimrc").exists();
    let log_exists = dirs.local.join("log.txt").exists();
    assert_eq!(bash_exists, false);
    assert_eq!(git_exists, false);
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_exists, false);
    assert_eq!(log_exists, false);
}
