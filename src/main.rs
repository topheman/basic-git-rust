#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use std::io::Read;
use std::path::{PathBuf, Path};
use anyhow::bail;
use clap::{Command, Arg};
use anyhow::anyhow;
// use flate2::read::GzDecoder;

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
                        let resolved_path = std::env::current_dir().expect("Expect retrieving current working directory").as_path().join(init_path);
                        resolved_path.clone()
                    }
                     else {
                        init_path.to_path_buf()
                    };
                    // todo git real implementation allows to init git repo inside a git repo - remove this check ?
                    if let Ok(git_root) = find_git_root(&path) {
                        bail!("The folder is already versionned by git in {}", git_root.display());
                    }
                    println!("Initializing {}", path.join(".git").display());
                    // todo better errors such as "No such file or directory"
                    fs::create_dir(path.join(".git"))?;
                    fs::create_dir(path.join(".git").join("objects"))?;
                    fs::create_dir(path.join(".git").join("refs"))?;
                    fs::write(path.join(".git").join("HEAD"), "ref: refs/heads/master\n")?;
                    println!("Initialized git directory");
                    return Ok(())
                }
                _ => unreachable!()
            }
        }
        Some(("cat-file", submatches)) => {
            match submatches.get_one::<String>("blob_sha") {
                Some(blob_sha) => {
                    if blob_sha.len() != 40 {
                        return Err(anyhow!("sha must contain 40 characters, passed {}", blob_sha));
                    }
                    // todo make a macro out of it ?
                    if let Ok(git_root) = find_git_root_from_cwd() {
                        let target_path = git_root
                            .join("objects")
                            .join(blob_sha.get(0..2).unwrap())
                            .join(blob_sha.get(2..).unwrap());
                        if let Ok(file_content) = fs::read_to_string(target_path.clone()) {
                            let mut decoder = flate2::read::ZlibDecoder::new(&*file_content.as_bytes());
                            let mut output = String::new();
                            decoder.read_to_string(&mut output).unwrap();
                            println!("{}", output);
                        }
                        else {
                            bail!("{} not found in {}", blob_sha, target_path.display());
                        }
                        println!("{}", blob_sha,);
                        println!(
                            "{}{}",
                            blob_sha.get(0..2).unwrap(),
                            blob_sha.get(2..).unwrap()
                        );
                    }
                    return anyhow::Ok(());
                }
                None => {
                    eprintln!("You must provide a sha");
                    return anyhow::Ok(())
                }
            }
        }
        Some((ext, _)) => {
            println!("unknown command \"{}\"", ext);
            anyhow::Ok(())
        }
        _ => unreachable!()
    }
}
