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

#[derive(PartialEq, Debug)]
enum GitRevSpec {
    CommitSha(String),
    RefPath(String),
}

#[derive(PartialEq, Debug)]
enum GitRevSpecPath {
    CommitSha(PathBuf),
    RefPath(PathBuf),
}

fn parse_git_rev_spec(git_rev_spec: String) -> GitRevSpec {
    return GitRevSpec::CommitSha("".to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    const GIT_HEAD: &str = "ref: refs/heads/master";
    const GIT_REFS_HEADS_MASTER: &str = "af648df27488d558e794eb1e25304a90930d9d38";
    const GIT_REFS_HEADS_FEAT_FOO: &str = "eea961ba163d275c90fd6ba57d70754809b428a1";
    const GIT_REFS_TAGS_V0_1_0: &str = "b82608c0bb54a84ae7b3d38112ccf1cb50aebe8d";

    fn mock_read(path: &PathBuf) -> std::io::Result<Vec<u8>> {
        match path.to_str() {
            Some(".git/HEAD") => Ok(GIT_HEAD.as_bytes().to_owned()),
            Some(".git/refs/heads/master") => Ok(GIT_REFS_HEADS_MASTER.as_bytes().to_owned()),
            Some(".git/refs/heads/feat/foo") => Ok(GIT_REFS_HEADS_FEAT_FOO.as_bytes().to_owned()),
            Some(".git/refs/heads/tags/v0.1.0") => Ok(GIT_REFS_TAGS_V0_1_0.as_bytes().to_owned()),
            Some(_) | None => Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found - {:?}", path),
            )),
        }
    }

    #[test]
    fn parse_git_rev_spec_commit_sha() {
        assert_eq!(
            parse_git_rev_spec("af648df27488d558e794eb1e25304a90930d9d38".to_string()),
            GitRevSpec::CommitSha("af648df27488d558e794eb1e25304a90930d9d38".to_string())
        )
    }

    #[test]
    fn parse_git_rev_spec_ref_path() {
        assert_eq!(
            parse_git_rev_spec("ref: refs/heads/master".to_string()),
            GitRevSpec::RefPath("refs/heads/master".to_string())
        )
    }
}
