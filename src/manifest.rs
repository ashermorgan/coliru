use serde::Deserialize;
use serde_yaml;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Deserialize)]
pub struct CopyLinkOptions {
    pub src: String,
    pub dst: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct RunOptions {
    pub src: String,

    #[serde(default)]
    pub prefix: String,

    #[serde(default)]
    pub postfix: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Step {
    #[serde(default)]
    pub copy: Vec<CopyLinkOptions>,

    #[serde(default)]
    pub link: Vec<CopyLinkOptions>,

    #[serde(default)]
    pub run: Vec<RunOptions>,

    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, PartialEq, Deserialize)]
struct RawManifest {
    steps: Vec<Step>,
}

#[derive(Debug, PartialEq)]
pub struct Manifest {
    pub steps: Vec<Step>,
    pub base_dir: PathBuf,
}

/// Parse a coliru YAML manifest file
pub fn parse_manifest_file(path: &Path) -> Result<Manifest, String> {
    let raw_str = read_to_string(path).map_err(|why| why.to_string())?;
    let raw_manifest = serde_yaml::from_str::<RawManifest>(&raw_str)
        .map_err(|why| why.to_string())?;
    let base_dir = match path.parent() {
        None => &Path::new("."),
        Some(p) => if p == Path::new("") { &Path::new(".") } else { p },
    };

    Ok(Manifest {
        steps: raw_manifest.steps,
        base_dir: base_dir.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn parse_manifest_file_missing() {
        let expected = "No such file or directory (os error 2)";
        let actual = parse_manifest_file(Path::new("examples/missing.yml"));
        assert_eq!(actual, Err(String::from(expected)));
    }

    #[test]
    fn parse_manifest_file_invalid() {
        let expected = "steps[1].copy[0]: missing field `src` at line 12 column 7";
        let actual = parse_manifest_file(Path::new("examples/invalid.yml"));
        assert_eq!(actual, Err(String::from(expected)));
    }

    #[test]
    fn parse_manifest_file_valid() {
        let expected = Manifest {
            steps: vec![
                Step {
                    copy: vec![
                        CopyLinkOptions {
                            src: String::from("foo"),
                            dst: String::from("/foo"),
                        },
                    ],
                    link: vec![
                        CopyLinkOptions {
                            src: String::from("foo"),
                            dst: String::from("~/foo"),
                        },
                        CopyLinkOptions {
                            src: String::from("bar"),
                            dst: String::from("~/test/bar"),
                        },
                    ],
                    run: vec![],
                    tags: vec![String::from("a"), String::from("b")],
                },
                Step {
                    copy: vec![],
                    link: vec![],
                    run: vec![
                        RunOptions {
                            src: String::from("baz"),
                            prefix: String::from(""),
                            postfix: String::from("arg1 $COLIRU_RULES arg2"),
                        },
                    ],
                    tags: vec![String::from("c")],
                }
            ],
            base_dir: PathBuf::from("examples"),
        };
        let actual = parse_manifest_file(Path::new("examples/manifest.yml"));
        assert_eq!(actual, Ok(expected));
    }
}
