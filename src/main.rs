#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::path::{PathBuf, Path};
use clap::{Command, Arg};
use anyhow::anyhow;

fn cli() -> Command {
    Command::new("mygit")
        .about("A toy git implementation in rust")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("init")
                .arg(
                    Arg::new("init_path").default_value(".").help("Init path")
                )
                .about("Create an empty Git repository or reinitialize an existing one")
        )
        .subcommand(
            Command::new("cat-file")
            .about("Provide content or type and size information for repository objects")
            .arg(
                Arg::new("blob_sha").short('p').help("Pretty-print the contents of <object> based on its type.")
            )
        )
}

fn find_git_root(dir: &PathBuf) -> anyhow::Result<PathBuf> {
    let mut git_root = dir.as_path();
    loop {
        match std::fs::metadata(git_root.join(".git")) {
            Ok(m) if m.is_dir() => {
                break;
            }
            Ok(_) | Err(_) => {
                // try one level up
                git_root = git_root.parent().ok_or_else(|| anyhow!("Failed to detect git directory"))?;
            }
        }
    }
    Ok(git_root.to_path_buf())
}

fn main() -> anyhow::Result<(), anyhow::Error> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("init", submatches)) => {
            submatches.get_one::<String>("init_path").and_then(|init_path| {
                let init_path = Path::new(init_path);
                let path = if !(init_path).is_absolute() {
                    let resolved_path = std::env::current_dir().expect("Expect retrieving current working directory").as_path().join(init_path);
                    resolved_path.clone()
                }
                 else {
                    init_path.to_path_buf()
                };
                // todo git real implementation allows to init git repo inside a git repo - remove this check ?
                if let Ok(git_root) = find_git_root(&path) {
                    println!("The folder is already versionned by git in {}", git_root.display());
                    return Some(());
                }
                println!("Initializing {}", path.join(".git").display());
                // todo better errors such as "No such file or directory"
                fs::create_dir(path.join(".git")).unwrap();
                fs::create_dir(path.join(".git").join("objects")).unwrap();
                fs::create_dir(path.join(".git").join("refs")).unwrap();
                fs::write(path.join(".git").join("HEAD"), "ref: refs/heads/master\n").unwrap();
                println!("Initialized git directory");
                Some(())
            });
            anyhow::Ok(())
        }
        Some(("cat-file", submatches)) => {
            submatches.get_one::<String>("blob_sha").and_then(|blob_sha| {
                if blob_sha.len() != 40 {
                    println!("sha must contain 40 characters");
                    return None;
                }
                println!("{}", blob_sha);
                return Some(blob_sha)
            });
            anyhow::Ok(())
        }
        Some((ext, _)) => {
            println!("unknown command \"{}\"", ext);
            anyhow::Ok(())
        }
        _ => unreachable!()
    }
}
