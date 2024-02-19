use anyhow::anyhow;
use bstr::ByteSlice;
use std::path::PathBuf;

use super::io::file_exists;

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

#[derive(PartialEq)]
enum GitRevSpecType {
    Head,
    CommitSha,
    BranchName,
    TagName,
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

impl GitRevSpecParsed {
    fn to_path_buf(
        &self,
        git_rev_spec_type: GitRevSpecType,
        git_root: &PathBuf,
    ) -> Result<PathBuf, anyhow::Error> {
        match git_rev_spec_type {
            GitRevSpecType::Head => {
                if self.value == "HEAD" {
                    let target_path = git_root.join("HEAD");
                    return Ok(target_path);
                } else {
                    return Err(anyhow!("Not a HEAD"));
                }
            }
            GitRevSpecType::CommitSha => {
                if self.value.len() != 40 {
                    return Err(anyhow!("Not a commitsha"));
                }
                let value = &self.value;
                let target_path = value
                    .get(0..2)
                    .and_then(|first_fragment| {
                        value
                            .get(2..)
                            .and_then(|second_fragment| Some((first_fragment, second_fragment)))
                    })
                    .and_then(|(first_fragment, second_fragment)| {
                        let target_path = PathBuf::new();
                        let target_path = target_path
                            .join("objects")
                            .join(first_fragment)
                            .join(second_fragment);
                        return Some(target_path);
                    });
                if target_path.is_some() {
                    return Ok(git_root.join(target_path.unwrap()));
                }
                return Err(anyhow!("Not a commitsha"));
            }
            GitRevSpecType::BranchName | GitRevSpecType::TagName => {
                let mut target_path = git_root.join("refs");
                if git_rev_spec_type == GitRevSpecType::BranchName {
                    target_path.push("heads");
                }
                if git_rev_spec_type == GitRevSpecType::TagName {
                    target_path.push("tags");
                }
                let target_path =
                    parser_helpers::git_rev_spec_value_into_path_buf(&self.value, target_path);
                return Ok(target_path);
            }
        }
    }
}

#[cfg(test)]
mod test_git_rev_spec_parsed_impl {
    use super::*;

    #[test]
    fn test_to_path_buffer_head_ok() {
        let git_rev_spec_parsed = GitRevSpecParsed {
            value: "HEAD".to_string(),
            modifier: None,
        };
        let git_root = PathBuf::new().join(".git");
        let resolved_path = PathBuf::new().join(".git").join("HEAD");
        assert_eq!(
            git_rev_spec_parsed
                .to_path_buf(GitRevSpecType::Head, &git_root)
                .unwrap(),
            resolved_path
        )
    }

    #[test]
    fn test_to_path_buffer_commit_ok() {
        let git_rev_spec_parsed = GitRevSpecParsed {
            value: "af648df27488d558e794eb1e25304a90930d9d38".to_string(),
            modifier: None,
        };
        let git_root = PathBuf::new().join(".git");
        let resolved_path = PathBuf::new();
        let resolved_path = PathBuf::new()
            .join(".git")
            .join("objects")
            .join("af")
            .join("648df27488d558e794eb1e25304a90930d9d38");
        assert_eq!(
            git_rev_spec_parsed
                .to_path_buf(GitRevSpecType::CommitSha, &git_root)
                .unwrap(),
            resolved_path
        )
    }

    #[test]
    fn test_to_path_buffer_commit_ko_not_correct_length() {
        let git_rev_spec_parsed = GitRevSpecParsed {
            value: "af648df27488d558e794eb".to_string(),
            modifier: None,
        };
        let git_root = PathBuf::new().join(".git");

        let error = git_rev_spec_parsed
            .to_path_buf(GitRevSpecType::CommitSha, &git_root)
            .unwrap_err();

        assert_eq!(format!("{}", error.root_cause()), "Not a commitsha");
    }

    #[test]
    fn test_to_path_buffer_branch_no_slash() {
        let git_rev_spec_parsed = GitRevSpecParsed {
            value: "master".to_string(),
            modifier: None,
        };
        let git_root = PathBuf::new().join(".git");
        let resolved_path = PathBuf::new()
            .join(".git")
            .join("refs")
            .join("heads")
            .join("master");
        assert_eq!(
            git_rev_spec_parsed
                .to_path_buf(GitRevSpecType::BranchName, &git_root)
                .unwrap(),
            resolved_path
        )
    }

    #[test]
    fn test_to_path_buffer_branch_multiple_slashes() {
        let git_rev_spec_parsed = GitRevSpecParsed {
            value: "feat/foo/bar".to_string(),
            modifier: None,
        };
        let git_root = PathBuf::new().join(".git");
        let resolved_path = PathBuf::new();
        let resolved_path = PathBuf::new()
            .join(".git")
            .join("refs")
            .join("heads")
            .join("feat")
            .join("foo")
            .join("bar");
        assert_eq!(
            git_rev_spec_parsed
                .to_path_buf(GitRevSpecType::BranchName, &git_root)
                .unwrap(),
            resolved_path
        )
    }

    #[test]
    fn test_to_path_buffer_tag_no_slash() {
        let git_rev_spec_parsed = GitRevSpecParsed {
            value: "v1".to_string(),
            modifier: None,
        };
        let git_root = PathBuf::new().join(".git");
        let resolved_path = PathBuf::new()
            .join(".git")
            .join("refs")
            .join("tags")
            .join("v1");
        assert_eq!(
            git_rev_spec_parsed
                .to_path_buf(GitRevSpecType::TagName, &git_root)
                .unwrap(),
            resolved_path
        )
    }

    #[test]
    fn test_to_path_buffer_tag_multiple_slashes() {
        let git_rev_spec_parsed = GitRevSpecParsed {
            value: "v1/foo/bar".to_string(),
            modifier: None,
        };
        let git_root = PathBuf::new().join(".git");
        let resolved_path = PathBuf::new()
            .join(".git")
            .join("refs")
            .join("tags")
            .join("v1")
            .join("foo")
            .join("bar");
        assert_eq!(
            git_rev_spec_parsed
                .to_path_buf(GitRevSpecType::TagName, &git_root)
                .unwrap(),
            resolved_path
        )
    }
}

fn parse_git_rev_spec(
    git_rev_spec: &str,
) -> Result<GitRevSpecParsed, nom::Err<nom::error::Error<&str>>> {
    match parser_helpers::take_rev_spec(git_rev_spec) {
        Ok((modifier, value)) => {
            return Ok(GitRevSpecParsed {
                value: value.to_string(),
                modifier: if !modifier.is_empty() {
                    Some(modifier.to_string())
                } else {
                    None
                },
            })
        }
        Err(e) => return Err(e),
    }
}

mod parser_helpers {
    use std::path::PathBuf;

    use nom::{bytes::complete::*, combinator::*, IResult};

    fn is_caret(c: char) -> bool {
        c == '^'
    }
    fn is_tilde(c: char) -> bool {
        c == '~'
    }
    fn is_at(c: char) -> bool {
        c == '@'
    }
    pub fn take_rev_spec(input: &str) -> IResult<&str, &str> {
        take_while(|c| !is_caret(c) && !is_tilde(c) && !is_at(c))(input)
    }
    pub fn git_rev_spec_value_into_path_buf(input: &str, mut root_path_buf: PathBuf) -> PathBuf {
        let fragments: Vec<&str> = input.split("/").collect();
        for fragment in fragments.iter() {
            root_path_buf.push(fragment);
        }
        return root_path_buf;
    }

    #[cfg(test)]
    mod tests_parser_helpers {
        use bstr::ByteSlice;
        use nom::AsBytes;

        use super::*;

        #[test]
        fn test_take_rev_spec_carets() {
            assert_eq!(
                take_rev_spec("feat/master^^").unwrap(),
                ("^^", "feat/master")
            );
        }
        #[test]
        fn test_take_rev_spec_carets_and_tilde() {
            assert_eq!(
                take_rev_spec("feat/master^^~^~").unwrap(),
                ("^^~^~", "feat/master")
            );
        }
        #[test]
        fn test_take_rev_spec_carets_tilde_with_number() {
            assert_eq!(
                take_rev_spec("feat/master^~3^^").unwrap(),
                ("^~3^^", "feat/master")
            );
        }
        #[test]
        fn test_take_rev_spec_at() {
            assert_eq!(
                take_rev_spec("feat/master@{3}").unwrap(),
                ("@{3}", "feat/master")
            );
        }
    }
}

// #[cfg(all(test, not(feature = "ignore_tests")))]
#[cfg(test)]
mod tests_git_rev_spec_parsed {
    use super::{parse_git_rev_spec, GitRevSpecParsed};

    #[test]
    fn test_parse_parse_git_rev_spec() {
        assert_eq!(
            parse_git_rev_spec("HEAD").unwrap(),
            GitRevSpecParsed {
                value: "HEAD".to_string(),
                modifier: None
            }
        );
    }

    #[test]
    fn test_parse_parse_git_rev_spec_caret() {
        assert_eq!(
            parse_git_rev_spec("HEAD^").unwrap(),
            GitRevSpecParsed {
                value: "HEAD".to_string(),
                modifier: Some("^".to_string())
            }
        );
    }

    #[test]
    fn test_parse_parse_git_rev_spec_double_caret() {
        assert_eq!(
            parse_git_rev_spec("HEAD^^").unwrap(),
            GitRevSpecParsed {
                value: "HEAD".to_string(),
                modifier: Some("^^".to_string())
            }
        );
    }

    #[test]
    fn test_parse_parse_git_rev_spec_tilde() {
        assert_eq!(
            parse_git_rev_spec("HEAD~").unwrap(),
            GitRevSpecParsed {
                value: "HEAD".to_string(),
                modifier: Some("~".to_string())
            }
        );
    }

    #[test]
    fn test_parse_parse_git_rev_spec_double_tilde() {
        assert_eq!(
            parse_git_rev_spec("HEAD~~").unwrap(),
            GitRevSpecParsed {
                value: "HEAD".to_string(),
                modifier: Some("~~".to_string())
            }
        );
    }

    #[test]
    fn test_parse_parse_git_rev_spec_at() {
        assert_eq!(
            parse_git_rev_spec("HEAD@{5}").unwrap(),
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
            parse_git_rev_spec("feat/@toto@{3}").unwrap(), // we won't support shis use case - too complicated / too little
            GitRevSpecParsed {
                value: "feat/@toto".to_string(),
                modifier: Some("@{3}".to_string())
            }
        );
    }

    #[test]
    fn test_parse_parse_git_rev_spec_combine_modifiers_1() {
        assert_eq!(
            parse_git_rev_spec("HEAD@{5}~2^").unwrap(),
            GitRevSpecParsed {
                value: "HEAD".to_string(),
                modifier: Some("@{5}~2^".to_string())
            }
        );
    }

    #[test]
    fn test_parse_parse_git_rev_spec_combine_modifiers_2() {
        assert_eq!(
            parse_git_rev_spec("HEAD^~2@{5}").unwrap(), // WARN this one is impossible, you can't have `@{n}` at the end - to handle when parsing modifiers
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
enum GitRevSpecResolved {
    Match(String),
    Ambiguous(String),
    NotFound,
}

fn resolve_git_rev_spec_parsed(
    git_rev_spec_parsed: GitRevSpecParsed,
    git_root: &PathBuf,
    read_as_string: fn(&PathBuf) -> Result<String, anyhow::Error>,
    file_exists: fn(&PathBuf) -> bool,
) -> GitRevSpecResolved {
    let mut matches: Vec<String> = Vec::new();
    // todo missing `.git` (git_root)
    if let Ok(path_buf_head) = git_rev_spec_parsed.to_path_buf(GitRevSpecType::Head, git_root) {
        if let Ok(head_content) = read_as_string(&path_buf_head) {
            return GitRevSpecResolved::Match(head_content);
        }
    }
    if let Ok(path_buf_commit) =
        git_rev_spec_parsed.to_path_buf(GitRevSpecType::CommitSha, git_root)
    {
        if file_exists(&path_buf_commit) {
            return GitRevSpecResolved::Match(git_rev_spec_parsed.value);
        }
    }
    if let Ok(path_buf_branch) =
        git_rev_spec_parsed.to_path_buf(GitRevSpecType::BranchName, git_root)
    {
        println!("branch {:?}", path_buf_branch);
        if let Ok(branch_content) = read_as_string(&path_buf_branch) {
            matches.push(branch_content);
        }
    }
    if let Ok(path_buf_tag) = git_rev_spec_parsed.to_path_buf(GitRevSpecType::TagName, git_root) {
        println!("tag {:?}", path_buf_tag);
        if let Ok(tag_content) = read_as_string(&path_buf_tag) {
            matches.push(tag_content);
        }
    }
    match matches.len() {
        2 => GitRevSpecResolved::Ambiguous(matches.get(1).unwrap().to_owned()),
        1 => GitRevSpecResolved::Match(matches.get(0).unwrap().to_owned()),
        0 => GitRevSpecResolved::NotFound,
        _ => {
            unreachable!()
        }
    }
}

fn try_resolve_git_rev_spec_parsed_commit(
    git_rev_spec_parsed: GitRevSpecParsed,
    file_exists: fn(&PathBuf) -> bool,
) {
}

#[cfg(all(test, not(feature = "ignore_tests")))]
mod tests_resolve_git_rev_spec_parsed {
    use std::path::PathBuf;

    use super::{resolve_git_rev_spec_parsed, GitRevSpecParsed, GitRevSpecResolved};

    const GIT_HEAD: &str = "ref: refs/heads/master";
    const GIT_REFS_HEADS_MASTER: &str = "af648df27488d558e794eb1e25304a90930d9d38";
    const GIT_REFS_HEADS_FEAT_FOO: &str = "eea961ba163d275c90fd6ba57d70754809b428a1";
    const GIT_REFS_TAGS_V0_1_0: &str = "b82608c0bb54a84ae7b3d38112ccf1cb50aebe8d";
    const GIT_REFS_HEADS_AMBIGUOUS: &str = "4269cb47c9e9afc08092a4344c7ec7c9545c51ee";
    const GIT_REFS_TAGS_AMBIGUOUS: &str = "ef916326703d7dcd9cab6b9ba6bb4793912508af";
    const MOCK_COMMIT_CONTENT: &str = "mock commit content"; // todo should be formatted+compressed but not necessary for the test

    fn mock_read_as_string(path: &std::path::PathBuf) -> Result<String, anyhow::Error> {
        match path.to_str() {
            Some(".git/HEAD") => Ok(GIT_HEAD.to_string()),
            Some(".git/refs/heads/master") => Ok(GIT_REFS_HEADS_MASTER.to_string()),
            Some(".git/refs/heads/feat/foo") => Ok(GIT_REFS_HEADS_FEAT_FOO.to_string()),
            Some(".git/refs/tags/v0.1.0") => Ok(GIT_REFS_TAGS_V0_1_0.to_string()),
            Some(".git/refs/heads/ambiguous") => Ok(GIT_REFS_HEADS_AMBIGUOUS.to_string()),
            Some(".git/refs/tags/ambiguous") => Ok(GIT_REFS_TAGS_AMBIGUOUS.to_string()),
            Some(".git/objects/af/648df27488d558e794eb1e25304a90930d9d38") => {
                Ok(MOCK_COMMIT_CONTENT.to_string())
            }
            Some(".git/objects/ee/a961ba163d275c90fd6ba57d70754809b428a1") => {
                Ok(MOCK_COMMIT_CONTENT.to_string())
            }
            Some(".git/objects/b8/2608c0bb54a84ae7b3d38112ccf1cb50aebe8d") => {
                Ok(MOCK_COMMIT_CONTENT.to_string())
            }
            Some(_) | None => Err(anyhow::anyhow!("File not found - {:?}", path)),
        }
    }
    fn mock_file_exits(path: &std::path::PathBuf) -> bool {
        match mock_read_as_string(path) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    #[test]
    fn test_resolve_git_rev_spec_parsed_commit_sha() {
        let git_root = PathBuf::new().join(".git");
        assert_eq!(
            resolve_git_rev_spec_parsed(
                GitRevSpecParsed {
                    value: GIT_REFS_HEADS_MASTER.to_string(),
                    modifier: None,
                },
                &git_root,
                mock_read_as_string,
                mock_file_exits
            ),
            GitRevSpecResolved::Match(GIT_REFS_HEADS_MASTER.to_string())
        )
    }

    #[test]
    fn test_resolve_git_rev_spec_parsed_head() {
        let git_root = PathBuf::new().join(".git");
        assert_eq!(
            resolve_git_rev_spec_parsed(
                GitRevSpecParsed {
                    value: "HEAD".to_string(),
                    modifier: None,
                },
                &git_root,
                mock_read_as_string,
                mock_file_exits
            ),
            GitRevSpecResolved::Match(GIT_REFS_HEADS_MASTER.to_string())
        )
    }

    #[test]
    fn test_resolve_git_rev_spec_parsed_feat_foo() {
        let git_root = PathBuf::new().join(".git");
        assert_eq!(
            resolve_git_rev_spec_parsed(
                GitRevSpecParsed {
                    value: "feat/foo".to_string(),
                    modifier: None,
                },
                &git_root,
                mock_read_as_string,
                mock_file_exits
            ),
            GitRevSpecResolved::Match(GIT_REFS_HEADS_FEAT_FOO.to_string())
        )
    }

    #[test]
    fn test_resolve_git_rev_spec_parsed_tag_v_010() {
        let git_root = PathBuf::new().join(".git");
        assert_eq!(
            resolve_git_rev_spec_parsed(
                GitRevSpecParsed {
                    value: "v0.1.0".to_string(),
                    modifier: None,
                },
                &git_root,
                mock_read_as_string,
                mock_file_exits
            ),
            GitRevSpecResolved::Match(GIT_REFS_TAGS_V0_1_0.to_string())
        )
    }
}
