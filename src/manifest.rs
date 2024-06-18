use serde::{Serialize, Deserialize};
use serde_yaml;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CopyOptions {
    pub src: PathBuf,
    pub dst: PathBuf,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Step {
    pub copy: Vec<CopyOptions>,
    pub tags: Vec<String>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub steps: Vec<Step>,
}

/// Parse a coliru YAML manifest file
pub fn parse_manifest_file(path: &Path) -> Result<Manifest, String> {
    match read_to_string(path) {
        Ok(raw) => match serde_yaml::from_str::<Manifest>(&raw) {
            Ok(result) => Ok(result),
            Err(why) => Err(why.to_string()),
        }
        Err(why) => Err(why.to_string()),
    }
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
                            src: PathBuf::from("foo"),
                            dst: PathBuf::from("~/foo")
                        },
                        CopyOptions{
                            src: PathBuf::from("bar"),
                            dst: PathBuf::from("~/test/bar")
                        },
                    ],
                    tags: vec![String::from("a"), String::from("b")],
                },
                Step {
                    copy: vec![
                        CopyOptions{
                            src: PathBuf::from("baz"),
                            dst: PathBuf::from("/baz")
                        },
                    ],
                    tags: vec![String::from("b"), String::from("c")],
                }
            ],
        };
        let actual = parse_manifest_file(Path::new("examples/2.yml"));
        assert_eq!(actual, Ok(expected));
    }
}
