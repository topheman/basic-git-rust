use anyhow::anyhow;
use anyhow::bail;
use clap::{Arg, Command};
use nom::AsBytes;
use nom::Parser;
#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::path::{Path, PathBuf};

// mod object;
mod parser;

use parser::unpack_object;

use crate::parser::{split_at_code, GitObjectHeader};

fn cli() -> Command {
    Command::new("mygit")
        .about("A toy git implementation in rust")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("init")
                .arg(Arg::new("init_path").default_value(".").help("Init path"))
                .about("Create an empty Git repository or reinitialize an existing one"),
        )
        .subcommand(
            Command::new("cat-file")
                .about("Provide content or type and size information for repository objects")
                .arg(
                    Arg::new("blob_sha")
                        .short('p')
                        .help("Pretty-print the contents of <object> based on its type."),
                ),
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
                git_root = git_root
                    .parent()
                    .ok_or_else(|| anyhow!("Failed to detect git directory"))?;
            }
        }
    }
    Ok(git_root.join(".git").to_path_buf())
}

fn find_git_root_from_cwd() -> anyhow::Result<PathBuf> {
    let cwd = std::env::current_dir().expect("Expect retrieving current working directory");
    return find_git_root(&cwd);
}

fn main() -> anyhow::Result<(), anyhow::Error> {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("init", submatches)) => {
            match submatches.get_one::<String>("init_path") {
                Some(init_path) => {
                    let init_path = Path::new(init_path);
                    let path = if !(init_path).is_absolute() {
                        let resolved_path = std::env::current_dir()
                            .expect("Expect retrieving current working directory")
                            .as_path()
                            .join(init_path);
                        resolved_path.clone()
                    } else {
                        init_path.to_path_buf()
                    };
                    // todo git real implementation allows to init git repo inside a git repo - remove this check ?
                    if let Ok(git_root) = find_git_root(&path) {
                        bail!(
                            "The folder is already versionned by git in {}",
                            git_root.display()
                        );
                    }
                    println!("Initializing {}", path.join(".git").display());
                    // todo better errors such as "No such file or directory"
                    fs::create_dir(path.join(".git"))?;
                    fs::create_dir(path.join(".git").join("objects"))?;
                    fs::create_dir(path.join(".git").join("refs"))?;
                    fs::write(path.join(".git").join("HEAD"), "ref: refs/heads/master\n")?;
                    println!("Initialized git directory");
                    return Ok(());
                }
                _ => unreachable!(),
            }
        }
        Some(("cat-file", submatches)) => match submatches.get_one::<String>("blob_sha") {
            Some(blob_sha) => {
                if blob_sha.len() != 40 {
                    return Err(anyhow!(
                        "sha must contain 40 characters, passed {}",
                        blob_sha
                    ));
                }
                if let Ok(git_root) = find_git_root_from_cwd() {
                    let target_path = git_root
                        .join("objects")
                        .join(blob_sha.get(0..2).unwrap())
                        .join(blob_sha.get(2..).unwrap());
                    let bytes = fs::read(&target_path)?;
                    println!("compressed: {:?}", bytes);
                    let content = unpack_object(bytes)?;
                    {
                        let content2 = content.clone().leak(); // todo weird lifetime hack due to parse bellow ...
                        println!("unpacked: {:?}", content2);
                        let (_, (git_object_infos, _)) = split_at_code(0).parse(content2)?;
                        println!("git_object_infos: {:?}", git_object_infos);
                        let git_object_header = GitObjectHeader::from_vec(&git_object_infos)?;
                        println!("git_object_header: {:?}", git_object_header);
                    }
                    println!("{}", std::str::from_utf8(content.as_bytes()).unwrap());
                }
                return anyhow::Ok(());
            }
            None => {
                eprintln!("You must provide a sha");
                return anyhow::Ok(());
            }
        },
        Some((ext, _)) => {
            println!("unknown command \"{}\"", ext);
            anyhow::Ok(())
        }
        _ => unreachable!(),
    }
}
