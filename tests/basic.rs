/// End to end tests that do not test general CLI behavior rather than specific
/// installation behavior

mod common;

use common::*;
use std::env::consts::EXE_SUFFIX;

#[test]
fn test_basic_help() {
    let (_dir, mut cmd) = setup_e2e("test_basic_help");
    cmd.arg("--help");
    let expected = format!("\
A minimal, flexible, dotfile installer

Usage: coliru{EXE_SUFFIX} [OPTIONS] <MANIFEST>

Arguments:
  <MANIFEST>  The path to the coliru YAML manifest file

Options:
  -t, --tag-rules [<TAG_RULES>...]  The set of tag rules to enforce
  -c, --copy                        Interpret link commands as copy commands
  -n, --dry-run                     Do a trial run without any permanent changes
      --host <HOST>                 Install dotfiles on another machine via SSH
  -h, --help                        Print help
  -V, --version                     Print version
");
    assert_eq!(stdout_to_string(&mut cmd), expected);
    assert_eq!(&stderr_to_string(&mut cmd), "");
}

#[test]
fn test_basic_empty_manifest() {
    let (dir, mut cmd) = setup_e2e("test_basic_empty_manifest");
    cmd.args(["manifest.yml"]);
    write_file(&dir.dir.join("manifest.yml"), "");

    let expected = "Error: missing field `steps`\n";
    assert_eq!(&stdout_to_string(&mut cmd), "");
    assert_eq!(&stderr_to_string(&mut cmd), expected);
}

#[test]
#[cfg(target_family = "unix")]
fn test_basic_missing_manifest() {
    let (_dir, mut cmd) = setup_e2e("test_basic_missing_manifest");
    cmd.args(["missing.yml"]);

    let expected = "Error: No such file or directory (os error 2)\n";
    assert_eq!(&stdout_to_string(&mut cmd), "");
    assert_eq!(&stderr_to_string(&mut cmd), expected);
}

#[test]
#[cfg(target_family = "windows")]
fn test_basic_missing_manifest() {
    let (_dir, mut cmd) = setup_e2e("test_basic_missing_manifest");
    cmd.args(["missing.yml"]);

    let expected = "Error: The system cannot find the file specified. \
                    (os error 2)\n";
    assert_eq!(&stdout_to_string(&mut cmd), "");
    assert_eq!(&stderr_to_string(&mut cmd), expected);
}

#[test]
fn test_basic_absolute_manifest() {
    let (dir, mut cmd) = setup_e2e("test_basic_absolute_manifest");
    let manifest_path = dir.dir.join("manifest.yml");
    cmd.args([&manifest_path.to_str().unwrap(), "--dry-run", "-t", "linux"]);
    copy_manifest(&dir.dir);

    let expected = "\
[1/3] Copy gitconfig to ~/.gitconfig.coliru (DRY RUN)
[2/3] Link bashrc to ~/.bashrc.coliru (DRY RUN)
[2/3] Link vimrc to ~/.vimrc.coliru (DRY RUN)
[2/3] Run sh script.sh arg1 linux (DRY RUN)
";
    assert_eq!(&stdout_to_string(&mut cmd), expected);
    assert_eq!(&stderr_to_string(&mut cmd), "");

    // Assert files are correctly copied/linked/run
    let bash_exists = dir.dir.join(".bashrc.coliru").exists();
    let git_exists = dir.dir.join(".gitconfig.coliru").exists();
    let vim1_exists = dir.dir.join(".vimrc.coliru").exists();
    let vim2_exists = dir.dir.join("_vimrc.coliru").exists();
    let log_exists = dir.dir.join("log.txt").exists();
    assert_eq!(bash_exists, false);
    assert_eq!(git_exists, false);
    assert_eq!(vim1_exists, false);
    assert_eq!(vim2_exists, false);
    assert_eq!(log_exists, false);
}
