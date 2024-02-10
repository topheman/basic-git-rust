use anyhow::anyhow;
use anyhow::bail;
use clap::{Arg, Command};
use std::fs;
use std::path::Path;

mod parser;
mod utils;

use parser::decompress_object;

use crate::parser::{GitObject, GitObjectHeader};
use crate::utils::io::{find_git_root, find_git_root_from_cwd};

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
                    let content = decompress_object(bytes)?;
                    match GitObject::from_vec(&content) {
                        Ok(GitObject {
                            header: GitObjectHeader::Tree(length),
                            raw_data,
                        }) => {
                            println!("header_infos: {:?}", GitObjectHeader::Tree(length));
                            println!("raw_data: {:?}", raw_data);
                            eprintln!("Tree object display not yet supported");
                            std::process::exit(1);
                        }
                        Ok(GitObject { header, raw_data }) => {
                            println!("header_infos: {:?}", header);
                            println!("raw_data: {:?}", raw_data);
                            println!("{}", std::str::from_utf8(&raw_data).unwrap());
                        }
                        Err(message) => {
                            eprintln!("Failed parsing object {}", message);
                        }
                    }
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
