#![allow(dead_code)]

use std::fmt::Debug;
use std::fs;
use std::io::Read;
use std::str;

use anyhow::{bail, Context, Result};
use flate2::read::ZlibDecoder;

type Hash = String;

pub trait Object: Debug {}

#[derive(Debug, Clone)]
struct Blob {
    content: String,
}

impl Object for Blob {}

#[derive(Debug, Clone)]
struct Tree {
    hash: Hash,
    unix_access_code: usize,
    file_name: String,
}

impl Object for Tree {}
impl Object for Vec<Tree> {}

#[derive(Debug, Clone)]
struct Commit {
    tree: Hash,
    parent: Option<Hash>,
    author: String,
    committer: String,
    gpgsig: Option<String>,
    message: String,
}

impl Object for Commit {}

#[derive(Debug, Clone)]
struct Tag {
    content: String,
}

impl Object for Tag {}

fn parse_object_header(header: &str) -> Result<(&str, usize)> {
    let (kind, length) = header
        .split_once(' ')
        .context("failed to split the header of the object")?;
    let length = length
        .parse::<usize>()
        .context("failed to parse the length of the object")?;

    Ok((kind, length))
}

pub fn parse_object_hash(hash: &str) -> Result<Box<dyn Object>> {
    let object_path = format!("./.git/objects/{}/{}", &hash[0..2], &hash[2..]);
    let object = fs::read(object_path).context("failed to read the object")?;

    let mut decoder = ZlibDecoder::new(object.as_slice());
    let mut buffer = Vec::new();
    decoder
        .read_to_end(&mut buffer)
        .context("failed to decompress the object")?;

    let mut buffer = buffer.into_iter();
    let header: Vec<u8> = buffer.by_ref().take_while(|c| *c != b'\0').collect();

    let header =
        str::from_utf8(&header).context("failed to parse the header of the object as utf8")?;
    let (kind, _length) = parse_object_header(header).context("failed to parse header")?;

    match kind {
        "blob" => Ok(Box::new(Blob {
            content: str::from_utf8(&buffer.collect::<Vec<u8>>())
                .context("failed to parse the content of the object as utf8")?
                .to_string(),
        })),
        "commit" => {
            let string = &buffer.collect::<Vec<u8>>();
            let string = str::from_utf8(string)
                .context("failed to parse the content of the object as utf8")?
                .to_string();

            let (metadata, message) = string
                .split_once("\n\n")
                .context("commit message should be separated by a new line")?;
            let message = message.trim().to_string();

            let mut metadata = metadata.split('\n');

            let mut tree = String::new();
            let mut parent = None;
            let mut author = String::new();
            let mut committer = String::new();
            let mut gpgsig = None;

            while let Some(s) = metadata.next() {
                let (kind, content) = s.split_once(' ').unwrap();
                match kind {
                    "tree" => {
                        tree = content.to_string();
                    }
                    "parent" => {
                        parent = Some(content.to_string());
                    }
                    "author" => {
                        author = content.to_string();
                    }
                    "committer" => {
                        committer = content.to_string();
                    }
                    "gpgsig" => {
                        let mut content = content.to_string();
                        let rest: String = metadata.by_ref().collect::<Vec<&str>>().join("\n");
                        content.push_str(&rest);
                        gpgsig = Some(content);
                    }
                    _ => {
                        bail!("unknown commit metadata")
                    }
                }
            }

            Ok(Box::new(Commit {
                tree,
                parent,
                author,
                committer,
                gpgsig,
                message,
            }))
        }
        "tag" => Ok(Box::new(Tag {
            content: str::from_utf8(&buffer.collect::<Vec<u8>>())
                .context("failed to parse the content of the object as utf8")?
                .to_string(),
        })),
        "tree" => {
            let mut content = Vec::new();

            loop {
                let unix_access_code: Vec<u8> =
                    buffer.by_ref().take_while(|c| *c != b' ').collect();
                let unix_access_code = str::from_utf8(&unix_access_code)?;

                // break when iterator starts yielding nothing
                if unix_access_code.is_empty() {
                    break;
                }

                let file_name: Vec<u8> = buffer.by_ref().take_while(|c| *c != b'\0').collect();
                let file_name = str::from_utf8(&file_name)?;

                let hash = buffer
                    .by_ref()
                    .take(20)
                    .map(|b| format!("{:02x}", b))
                    .collect::<Vec<String>>()
                    .join("");

                content.push(Tree {
                    unix_access_code: unix_access_code.parse()?,
                    file_name: file_name.to_string(),
                    hash: hash.to_string(),
                });
            }

            Ok(Box::new(content))
        }
        _ => bail!("unknown content type"),
    }
}

pub fn init_git_dir() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
    println!("Initialized git directory")
}
