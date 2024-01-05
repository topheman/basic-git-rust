#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;
use clap::{Command, Arg};

fn cli() -> Command {
    Command::new("mygit")
        .about("A toy git implementation in rust")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("init")
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

fn main() {
    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("init", _)) => {
            // todo accept path
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
            println!("Initialized git directory")
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
        }
        Some((ext, _)) => {
            println!("unknown command \"{}\"", ext);
        }
        _ => unreachable!()
    }
}
