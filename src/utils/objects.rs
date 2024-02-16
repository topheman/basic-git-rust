use std::path::PathBuf;

pub fn resolve_git_rev_spec(git_rev_spec: String) -> Result<String, anyhow::Error> {
    return Ok("".to_string());
}

fn resolve_head(
    git_root: &PathBuf,
    read: fn(&PathBuf) -> std::io::Result<Vec<u8>>,
) -> Result<Vec<u8>, anyhow::Error> {
    match read(&git_root.join(".git").join("HEAD")) {
        Ok(content) => {
            return Ok(content);
        }
        Err(e) => {
            return Err(e.into());
        }
    }
}

/// This is the result of parsing a git_rev_spec out of the command line like
/// - `"HEAD"` -> `GitRevSpecParsed { value: "HEAD", modifier: None }`
/// - `"HEAD^"` -> `GitRevSpecParsed { value: "HEAD", modifier: Some("^") }`
/// - `"master"` -> `GitRevSpecParsed { value: "master", modifier: None }`
/// - `"feat/foo"` -> `GitRevSpecParsed { value: "feat/foo", modifier: None }`
/// - `"develop^^"` -> `GitRevSpecParsed { value: "develop", modifier: Some("^^") }`
/// - `"v0.1.0~3"` -> `GitRevSpecParsed { value: "v0.1.0", modifier: Some("~3") }`
/// - `"HEAD@{5}"` -> `GitRevSpecParsed { value: "feat/foo", modifier: Some("@{5}") }`
#[derive(PartialEq, Debug)]
struct GitRevSpecParsed {
    value: String,
    modifier: Option<String>,
}

fn parse_git_rev_spec(git_rev_spec: String) -> GitRevSpecParsed {
    return GitRevSpecParsed {
        value: "foo".to_string(),
        modifier: None,
    };
}

#[cfg(test)]
mod tests_git_rev_spec_parsed {
    use super::{parse_git_rev_spec, GitRevSpecParsed};

    #[test]
    fn test_parse_parse_git_rev_spec() {
        assert_eq!(
            parse_git_rev_spec("HEAD".to_string()),
            GitRevSpecParsed {
                value: "HEAD".to_string(),
                modifier: None
            }
        );
    }

    #[test]
    fn test_parse_parse_git_rev_spec_caret() {
        assert_eq!(
            parse_git_rev_spec("HEAD^".to_string()),
            GitRevSpecParsed {
                value: "HEAD".to_string(),
                modifier: Some("^".to_string())
            }
        );
    }

    #[test]
    fn test_parse_parse_git_rev_spec_double_caret() {
        assert_eq!(
            parse_git_rev_spec("HEAD^^".to_string()),
            GitRevSpecParsed {
                value: "HEAD".to_string(),
                modifier: Some("^^".to_string())
            }
        );
    }

    #[test]
    fn test_parse_parse_git_rev_spec_tilde() {
        assert_eq!(
            parse_git_rev_spec("HEAD~".to_string()),
            GitRevSpecParsed {
                value: "HEAD".to_string(),
                modifier: Some("~".to_string())
            }
        );
    }

    #[test]
    fn test_parse_parse_git_rev_spec_double_tilde() {
        assert_eq!(
            parse_git_rev_spec("HEAD~~".to_string()),
            GitRevSpecParsed {
                value: "HEAD".to_string(),
                modifier: Some("~~".to_string())
            }
        );
    }

    #[test]
    fn test_parse_parse_git_rev_spec_at() {
        assert_eq!(
            parse_git_rev_spec("HEAD@{5}".to_string()),
            GitRevSpecParsed {
                value: "HEAD".to_string(),
                modifier: Some("@{5}".to_string())
            }
        );
    }

    #[ignore]
    #[test]
    fn test_parse_parse_git_rev_spec_at_with_at_in_branch_name() {
        assert_eq!(
            parse_git_rev_spec("feat/@toto@{3}".to_string()), // we won't support shis use case - too complicated / too little
            GitRevSpecParsed {
                value: "feat/@toto".to_string(),
                modifier: Some("@{3}".to_string())
            }
        );
    }

    #[test]
    fn test_parse_parse_git_rev_spec_combine_modifiers_1() {
        assert_eq!(
            parse_git_rev_spec("HEAD@{5}~2^".to_string()),
            GitRevSpecParsed {
                value: "HEAD".to_string(),
                modifier: Some("@{5}~2^".to_string())
            }
        );
    }

    #[test]
    fn test_parse_parse_git_rev_spec_combine_modifiers_2() {
        assert_eq!(
            parse_git_rev_spec("HEAD^~2@{5}".to_string()), // WARN this one is impossible, you can't have `@{n}` at the end - to handle when parsing modifiers
            GitRevSpecParsed {
                value: "HEAD".to_string(),
                modifier: Some("^~2@{5}".to_string())
            }
        );
    }
}

/// This is the result of resolving the `value` of a [`GitRevSpecParsed`], without applying the modifier
/// - `GitRevSpecParsed { value: "HEAD", modifier: None }` -> `GitRevSpecResolved { value: "somecommitsha" }`
/// - `GitRevSpecParsed { value: "HEAD", modifier: Some("^") }` -> `GitRevSpecResolved { value: "somecommitsha" }`
/// - `GitRevSpecParsed { value: "master", modifier: None }` -> `GitRevSpecResolved { value: "somecommitsha" }`
/// - `GitRevSpecParsed { value: "feat/foo", modifier: None }` -> `GitRevSpecResolved { value: "somecommitsha" }`
/// - `GitRevSpecParsed { value: "develop", modifier: Some("^^") }` -> `GitRevSpecResolved { value: "somecommitsha" }`
/// - `GitRevSpecParsed { value: "v0.1.0", modifier: Some("~3") }` -> `GitRevSpecResolved { value: "somecommitsha" }`
/// - `GitRevSpecParsed { value: "feat/foo", modifier: Some("@{5}") }` -> `GitRevSpecResolved { value: "somecommitsha" }`
#[derive(PartialEq, Debug)]
struct GitRevSpecResolved {
    value: String,
}

fn resolve_git_rev_spec_parsed(
    git_rev_spec_parsed: GitRevSpecParsed,
    read: fn(&PathBuf) -> std::io::Result<Vec<u8>>,
) -> Result<GitRevSpecResolved, anyhow::Error> {
    return Ok(GitRevSpecResolved {
        value: "af648df27488d558e794eb1e25304a90930d9d38".to_string(),
    });
}

#[cfg(test)]
mod tests_resolve_git_rev_spec_parsed {
    use super::{resolve_git_rev_spec_parsed, GitRevSpecParsed, GitRevSpecResolved};

    const GIT_HEAD: &str = "ref: refs/heads/master";
    const GIT_REFS_HEADS_MASTER: &str = "af648df27488d558e794eb1e25304a90930d9d38";
    const GIT_REFS_HEADS_FEAT_FOO: &str = "eea961ba163d275c90fd6ba57d70754809b428a1";
    const GIT_REFS_TAGS_V0_1_0: &str = "b82608c0bb54a84ae7b3d38112ccf1cb50aebe8d";
    const MOCK_COMMIT_CONTENT: &str = "mock commit content"; // todo should be formatted+compressed but not necessary for the test

    fn mock_read(path: &std::path::PathBuf) -> std::io::Result<Vec<u8>> {
        match path.to_str() {
            Some(".git/HEAD") => Ok(GIT_HEAD.as_bytes().to_owned()),
            Some(".git/refs/heads/master") => Ok(GIT_REFS_HEADS_MASTER.as_bytes().to_owned()),
            Some(".git/refs/heads/feat/foo") => Ok(GIT_REFS_HEADS_FEAT_FOO.as_bytes().to_owned()),
            Some(".git/refs/heads/tags/v0.1.0") => Ok(GIT_REFS_TAGS_V0_1_0.as_bytes().to_owned()),
            Some(".git/objects/af/648df27488d558e794eb1e25304a90930d9d38") => {
                Ok(MOCK_COMMIT_CONTENT.as_bytes().to_owned())
            }
            Some(".git/objects/ee/a961ba163d275c90fd6ba57d70754809b428a1") => {
                Ok(MOCK_COMMIT_CONTENT.as_bytes().to_owned())
            }
            Some(".git/objects/b8/2608c0bb54a84ae7b3d38112ccf1cb50aebe8d") => {
                Ok(MOCK_COMMIT_CONTENT.as_bytes().to_owned())
            }
            Some(_) | None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found - {:?}", path),
            )),
        }
    }

    #[test]
    fn test_resolve_git_rev_spec_parsed_commit_sha() {
        assert_eq!(
            resolve_git_rev_spec_parsed(
                GitRevSpecParsed {
                    value: GIT_REFS_HEADS_MASTER.to_string(),
                    modifier: None,
                },
                mock_read
            )
            .unwrap(),
            GitRevSpecResolved {
                value: GIT_REFS_HEADS_MASTER.to_string()
            }
        )
    }

    #[test]
    fn test_resolve_git_rev_spec_parsed_head() {
        assert_eq!(
            resolve_git_rev_spec_parsed(
                GitRevSpecParsed {
                    value: "HEAD".to_string(),
                    modifier: None,
                },
                mock_read
            )
            .unwrap(),
            GitRevSpecResolved {
                value: GIT_REFS_HEADS_MASTER.to_string()
            }
        )
    }

    #[test]
    fn test_resolve_git_rev_spec_parsed_feat_foo() {
        assert_eq!(
            resolve_git_rev_spec_parsed(
                GitRevSpecParsed {
                    value: "feat/foo".to_string(),
                    modifier: None,
                },
                mock_read
            )
            .unwrap(),
            GitRevSpecResolved {
                value: GIT_REFS_HEADS_FEAT_FOO.to_string()
            }
        )
    }

    #[test]
    fn test_resolve_git_rev_spec_parsed_tag_v_010() {
        assert_eq!(
            resolve_git_rev_spec_parsed(
                GitRevSpecParsed {
                    value: "v0.1.0".to_string(),
                    modifier: None,
                },
                mock_read
            )
            .unwrap(),
            GitRevSpecResolved {
                value: GIT_REFS_TAGS_V0_1_0.to_string()
            }
        )
    }
}
