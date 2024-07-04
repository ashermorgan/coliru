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

    #[test]
    #[cfg(target_family = "unix")]
    fn parse_manifest_file_missing() {
        let manifest_path = Path::new("examples/test/missing.yml");
        let expected = "No such file or directory (os error 2)";
        let actual = parse_manifest_file(manifest_path);
        assert_eq!(actual, Err(String::from(expected)));
    }

    #[test]
    #[cfg(target_family = "windows")]
    fn parse_manifest_file_missing() {
        let manifest_path = Path::new("examples/test/missing.yml");
        let exp = "The system cannot find the file specified. (os error 2)";
        let actual = parse_manifest_file(manifest_path);
        assert_eq!(actual, Err(String::from(exp)));
    }

    #[test]
    fn parse_manifest_file_invalid() {
        let manifest_path = Path::new("examples/test/invalid.yml");
        let exp = "steps[0].copy[0]: missing field `src` at line 5 column 7";
        let actual = parse_manifest_file(manifest_path);
        assert_eq!(actual, Err(String::from(exp)));
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
                    copy: vec![],
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
                    copy: vec![],
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
        assert_eq!(actual, Ok(expected));
    }
}
