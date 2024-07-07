//! Coliru manifest parsing

use anyhow::Result;
use serde::Deserialize;
use serde_yaml;
use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

/// The options for a copy or link command
#[derive(Debug, PartialEq, Deserialize)]
pub struct CopyLinkOptions {
    /// The source file (relative to the parent manifest file)
    pub src: String,

    /// The destination path (relative to the parent manifest file)
    pub dst: String,
}

/// The options for a run command
#[derive(Debug, PartialEq, Deserialize)]
pub struct RunOptions {
    /// The location of the script (relative to the parent manifest file)
    pub src: String,

    /// The optional shell command prefix
    #[serde(default)]
    pub prefix: String,

    /// The optional shell command postfix
    #[serde(default)]
    pub postfix: String,
}

/// A manifest step
#[derive(Debug, PartialEq, Deserialize)]
pub struct Step {
    /// The step's copy commands
    #[serde(default)]
    pub copy: Vec<CopyLinkOptions>,

    /// The step's link commands
    #[serde(default)]
    pub link: Vec<CopyLinkOptions>,

    /// The step's run commands
    #[serde(default)]
    pub run: Vec<RunOptions>,

    /// The step's tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A coliru manifest as it appears in a file, without the base_dir property
#[derive(Debug, PartialEq, Deserialize)]
struct RawManifest {

    /// The manifest steps
    steps: Vec<Step>,
}

/// A parsed coliru manifest
#[derive(Debug, PartialEq)]
pub struct Manifest {
    /// The manifest steps
    pub steps: Vec<Step>,

    /// The parent directory of the manifest file
    pub base_dir: PathBuf,
}

/// Parse a coliru YAML manifest file
///
/// ```
/// let manifest = parse_manifest_file(Path::new("manifest.yml"))?;
/// ```
pub fn parse_manifest_file(path: &Path) -> Result<Manifest> {
    let raw_str = read_to_string(path)?;
    let raw_manifest = serde_yaml::from_str::<RawManifest>(&raw_str)?;
    let base_dir = match path.parent() {
        None => &Path::new("."),
        Some(p) => if p == Path::new("") { &Path::new(".") } else { p },
    };

    Ok(Manifest {
        steps: raw_manifest.steps,
        base_dir: base_dir.to_path_buf(),
    })
}

/// Returns a sorted, de-duplicated vector of all tags in a manifest
pub fn get_tags(manifest: Manifest) -> Vec<String> {
    let mut tag_set: HashSet<String> = HashSet::new();

    for step in manifest.steps {
        for tag in step.tags {
            tag_set.insert(tag);
        }
    }

    let mut tags: Vec<String> = tag_set.iter().map(|s| s.to_owned()).collect();
    tags.sort();
    tags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_family = "unix")]
    fn parse_manifest_file_missing() {
        let manifest_path = Path::new("examples/test/missing.yml");
        let expected = "No such file or directory (os error 2)";
        let actual = parse_manifest_file(manifest_path);
        assert_eq!(actual.is_ok(), false);
        assert_eq!(actual.unwrap_err().to_string(), expected);
    }

    #[test]
    #[cfg(target_family = "windows")]
    fn parse_manifest_file_missing() {
        let manifest_path = Path::new("examples/test/missing.yml");
        let exp = "The system cannot find the file specified. (os error 2)";
        let actual = parse_manifest_file(manifest_path);
        assert_eq!(actual.is_ok(), false);
        assert_eq!(actual.unwrap_err().to_string(), exp);
    }

    #[test]
    fn parse_manifest_file_invalid() {
        let manifest_path = Path::new("examples/test/invalid.yml");
        let exp = "steps[0].copy[0]: missing field `src` at line 5 column 7";
        let actual = parse_manifest_file(manifest_path);
        assert_eq!(actual.is_ok(), false);
        assert_eq!(actual.unwrap_err().to_string(), exp);
    }

    #[test]
    fn parse_manifest_file_valid() {
        let manifest_path = Path::new("examples/test/manifest.yml");
        let expected = Manifest {
            steps: vec![
                Step {
                    copy: vec![
                        CopyLinkOptions {
                            src: String::from("gitconfig"),
                            dst: String::from("~/.gitconfig"),
                        },
                    ],
                    link: vec![],
                    run: vec![],
                    tags: vec![
                        String::from("windows"),
                        String::from("linux"),
                        String::from("macos")
                    ],
                },
                Step {
                    copy: vec![
                        CopyLinkOptions {
                            src: String::from("scripts/foo"),
                            dst: String::from("scripts/foo"),
                        },
                    ],
                    link: vec![
                        CopyLinkOptions {
                            src: String::from("bashrc"),
                            dst: String::from("~/.bashrc"),
                        },
                        CopyLinkOptions {
                            src: String::from("vimrc"),
                            dst: String::from("~/.vimrc"),
                        },
                    ],
                    run: vec![
                        RunOptions {
                            src: String::from("scripts/script.sh"),
                            prefix: String::from("sh"),
                            postfix: String::from("arg1 $COLIRU_RULES"),
                        },
                    ],
                    tags: vec![String::from("linux"), String::from("macos")],
                },
                Step {
                    copy: vec![
                        CopyLinkOptions {
                            src: String::from("scripts/foo"),
                            dst: String::from("scripts/foo"),
                        },
                    ],
                    link: vec![
                        CopyLinkOptions {
                            src: String::from("vimrc"),
                            dst: String::from("~/_vimrc"),
                        },
                    ],
                    run: vec![
                        RunOptions {
                            src: String::from("scripts/script.bat"),
                            prefix: String::from(""),
                            postfix: String::from("arg1 $COLIRU_RULES"),
                        },
                    ],
                    tags: vec![String::from("windows")],
                },
            ],
            base_dir: PathBuf::from("examples/test"),
        };
        let actual = parse_manifest_file(manifest_path);
        assert_eq!(actual.is_ok(), true);
        assert_eq!(actual.unwrap(), expected);
    }

    #[test]
    fn get_tags_basic() {
        let manifest_path = Path::new("examples/test/manifest.yml");
        let manifest = parse_manifest_file(manifest_path).unwrap();
        let expected = vec![
            String::from("linux"),
            String::from("macos"),
            String::from("windows"),
        ];
        let actual = get_tags(manifest);
        assert_eq!(actual, expected);
    }

    #[test]
    fn get_tags_empty() {
        let manifest = Manifest {
            steps: vec![],
            base_dir: PathBuf::from("examples/test/empty.yml"),
        };
        let expected: Vec<String> = vec![];
        let actual = get_tags(manifest);
        assert_eq!(actual, expected);
    }
}
