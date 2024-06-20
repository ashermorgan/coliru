use serde::Deserialize;
use serde_yaml;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Deserialize)]
pub struct CopyOptions {
    pub src: String,
    pub dst: String,
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Step {
    pub copy: Vec<CopyOptions>,
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
    let base_dir = path.parent().or_else(|| Some(&Path::new("."))).unwrap();

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
        let actual = parse_manifest_file(Path::new("examples/0.yml"));
        assert_eq!(actual, Err(String::from(expected)));
    }

    #[test]
    fn parse_manifest_file_invalid() {
        let expected = "steps[0]: missing field `copy` at line 4 column 5";
        let actual = parse_manifest_file(Path::new("examples/1.yml"));
        assert_eq!(actual, Err(String::from(expected)));
    }

    #[test]
    fn parse_manifest_file_valid() {
        let expected = Manifest {
            steps: vec![
                Step {
                    copy: vec![
                        CopyOptions{
                            src: String::from("foo"),
                            dst: String::from("~/foo"),
                        },
                        CopyOptions{
                            src: String::from("bar"),
                            dst: String::from("~/test/bar"),
                        },
                    ],
                    tags: vec![String::from("a"), String::from("b")],
                },
                Step {
                    copy: vec![
                        CopyOptions{
                            src: String::from("baz"),
                            dst: String::from("/baz"),
                        },
                    ],
                    tags: vec![String::from("b"), String::from("c")],
                }
            ],
            base_dir: PathBuf::from("examples"),
        };
        let actual = parse_manifest_file(Path::new("examples/2.yml"));
        assert_eq!(actual, Ok(expected));
    }
}
