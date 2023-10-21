use std::fs;
use std::io::Read;

use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Init,
    CatFile {
        #[arg(short = 'p')]
        hash: String,
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Command::Init => { init_git_repo() },
        Command::CatFile { hash } => { get_object_content(hash) },
    }
}

fn init_git_repo() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
    println!("Initialized git directory")
}

fn get_object_content(hash: &str) {
    let object_path = format!("./.git/objects/{}/{}", &hash[0..2], &hash[2..]);
    let object = fs::read(object_path).expect("object to exist");

    let mut d = ZlibDecoder::new(object.as_slice());
    let mut s = String::new();
    d.read_to_string(&mut s).unwrap();

    if let Some((_header, content)) = s.split_once('\0') {
        print!("{content}");
    }
}
