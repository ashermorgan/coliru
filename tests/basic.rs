mod common;

use common::{get_output, setup};

#[test]
fn test_help() {
    let (_dir, mut cmd) = setup("test_help");
    cmd.arg("--help");
    let expected = "\
A minimal, flexible, dotfile installer

Usage: coliru [OPTIONS] <MANIFEST>

Arguments:
  <MANIFEST>  The path to the coliru YAML manifest file

Options:
  -t, --tag-rules [<TAG_RULES>...]  The set of tag rules to enforce
  -c, --copy                        Interpret link commands as copy commands
  -n, --dry-run                     Do a trial run without any permanent changes
  -h, --help                        Print help
  -V, --version                     Print version
";
    assert_eq!(&get_output(&mut cmd), expected);
}
