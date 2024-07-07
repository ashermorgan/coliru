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
  -l, --list-tags              List available tags and quit without installing
  -n, --dry-run                Do a trial run without any permanent changes
      --host <HOST>            Install dotfiles on another machine over SSH
      --copy                   Interpret link commands as copy commands
      --no-color               Disable color output
  -h, --help                   Print help
  -V, --version                Print version

Examples:
  # List tags in manifest
  coliru manifest.yml --list-tags

  # Preview installation steps with tags matching A && (B || C) && !D
  coliru manifest.yml --tag-rules A B,C ^D --dry-run

  # Install dotfiles on local machine
  coliru manifest.yml --tag-rules A B,C ^D

  # Install dotfiles to user@hostname over SSH
  coliru manifest.yml --tag-rules A B,C ^D --host user@hostname
");
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, &expected);
    assert_eq!(exitcode, Some(0));
}

#[test]
fn test_basic_bad_arguments() {
    let (_dirs, mut cmd) = setup_e2e_local("test_basic_bad_arguments");
    cmd.args(["--foo", "bar"]);

    let expected = format!("\
error: unexpected argument '--foo' found

  tip: to pass '--foo' as a value, use '-- --foo'

Usage: coliru{EXE_SUFFIX} [OPTIONS] <MANIFEST>

For more information, try '--help'.
");
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, &expected);
    assert_eq!(&stdout, "");
    assert_eq!(exitcode, Some(2));
}

#[test]
fn test_basic_empty_manifest() {
    let (dirs, mut cmd) = setup_e2e_local("test_basic_empty_manifest");
    cmd.args(["manifest.yml"]);
    write_file(&dirs.local.join("manifest.yml"), "");

    let expected = "Error: Failed to parse manifest: missing field `steps`\n";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, expected);
    assert_eq!(&stdout, "");
    assert_eq!(exitcode, Some(2));
}

#[test]
#[cfg(target_family = "unix")]
fn test_basic_missing_manifest() {
    let (_dirs, mut cmd) = setup_e2e_local("test_basic_missing_manifest");
    cmd.args(["missing.yml"]);

    let expected = "Error: Failed to parse manifest: No such file or directory \
                    (os error 2)\n";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, expected);
    assert_eq!(&stdout, "");
    assert_eq!(exitcode, Some(2));
}

#[test]
#[cfg(target_family = "windows")]
fn test_basic_missing_manifest() {
    let (_dirs, mut cmd) = setup_e2e_local("test_basic_missing_manifest");
    cmd.args(["missing.yml"]);

    let expected = "Error: Failed to parse manifest: The system cannot find \
                    the file specified. (os error 2)\n";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, expected);
    assert_eq!(&stdout, "");
    assert_eq!(exitcode, Some(2));
}

#[test]
#[cfg(target_family = "unix")]
fn test_basic_absolute_manifest() {
    let (dirs, mut cmd) = setup_e2e_local("test_basic_absolute_manifest");
    let manifest_path = dirs.local.join("manifest.yml");
    cmd.args([&manifest_path.to_str().unwrap(), "--dry-run", "-t", "linux"]);

    let expected = "\
[1/3] Copy gitconfig to ~/.gitconfig (DRY RUN)
[2/3] Copy foo to foo (DRY RUN)
[2/3] Link bashrc to ~/.bashrc (DRY RUN)
[2/3] Link vimrc to ~/.vimrc (DRY RUN)
[2/3] Run sh script.sh arg1 linux (DRY RUN)
";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/linked/run
    let bash_exists = dirs.home.join(".bashrc").exists();
    let git_exists = dirs.home.join(".gitconfig").exists();
    let vim1_exists = dirs.home.join(".vimrc").exists();
    let vim2_exists = dirs.home.join("_vimrc").exists();
    let foo_exists = dirs.local.join("foo").exists();
    let log_exists = dirs.home.join("log.txt").exists();
    assert_eq!(bash_exists, false);
    assert_eq!(git_exists, false);
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_exists, false);
    assert_eq!(foo_exists, true);
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
[2/3] Copy foo to foo (DRY RUN)
[2/3] Link bashrc to .bashrc (DRY RUN)
[2/3] Link vimrc to .vimrc (DRY RUN)
[2/3] Run sh script.sh arg1 linux (DRY RUN)
";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, expected);
    assert_eq!(exitcode, Some(0));

    // Assert files are correctly copied/linked/run
    let bash_exists = dirs.local.join(".bashrc").exists();
    let git_exists = dirs.local.join(".gitconfig").exists();
    let vim1_exists = dirs.local.join(".vimrc").exists();
    let vim2_exists = dirs.local.join("_vimrc").exists();
    let foo_exists = dirs.local.join("foo").exists();
    let log_exists = dirs.local.join("log.txt").exists();
    assert_eq!(bash_exists, false);
    assert_eq!(git_exists, false);
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_exists, false);
    assert_eq!(foo_exists, true);
    assert_eq!(log_exists, false);
}

#[test]
fn test_basic_list_tags() {
    let (_dirs, mut cmd) = setup_e2e_local("test_basic_list_tags");
    cmd.args(["manifest.yml", "--list-tags", "-t", "windows"]);

    let expected = "\
linux
macos
windows
";
    let (stdout, stderr, exitcode) = run_command(&mut cmd);
    assert_eq!(&stderr, "");
    assert_eq!(&stdout, expected);
    assert_eq!(exitcode, Some(0));
}
