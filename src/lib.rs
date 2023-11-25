#![allow(dead_code)]

use std::fs;
use std::str;
use std::io::Read;
use std::fmt::Debug;

use anyhow::{Result, Context, bail};
use flate2::read::ZlibDecoder;

pub trait Object: Debug {}

#[derive(Debug, Clone)]
struct Blob {
    content: String,
}

impl Object for Blob {}

#[derive(Debug, Clone)]
struct Tree {
    hash: String,
    unix_access_code: usize,
    file_name: String,
}

impl Object for Tree {}
impl Object for Vec<Tree> {}

#[derive(Debug, Clone)]
struct Commit {
    content: String,
}

impl Object for Commit{}

#[derive(Debug, Clone)]
struct Tag {
    content: String,
}

impl Object for Tag {}

fn parse_object_header(header: &str) -> Result<(&str, usize)> {
    let (kind, length) = header.split_once(' ').context("failed to split the header of the object")?;
    let length = length.parse::<usize>().context("failed to parse the length of the object")?;

    Ok((kind, length))
}

pub fn parse_object_hash(hash: &str) -> Result<Box<dyn Object>> {
    let object_path = format!("./.git/objects/{}/{}", &hash[0..2], &hash[2..]);
    let object = fs::read(object_path).context("failed to read the object")?;

    let mut decoder = ZlibDecoder::new(object.as_slice());
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer).context("failed to decompress the object")?;

    let mut buffer = buffer.into_iter();
    let header: Vec<u8> = buffer.by_ref().take_while(|c| {*c != b'\0'}).collect();

    let header = str::from_utf8(&header).context("failed to parse the header of the object as utf8")?;
    let (kind, length) = parse_object_header(header).context("failed to parse header")?;

    match kind {
        "blob" => {
            Ok(Box::new(Blob {
                content: str::from_utf8(&buffer.collect::<Vec<u8>>())
                    .context("failed to parse the content of the object as utf8")?
                    .to_string()
            }))
        },
        "commit" => {
            Ok(Box::new(Commit {
                content: str::from_utf8(&buffer.collect::<Vec<u8>>())
                    .context("failed to parse the content of the object as utf8")?
                    .to_string()
            }))
        },
        "tag" => {
            Ok(Box::new(Tag {
                content: str::from_utf8(&buffer.collect::<Vec<u8>>())
                    .context("failed to parse the content of the object as utf8")?
                    .to_string()
            }))
        },
        "tree" => {
            let mut content = Vec::new();

            loop {
                let unix_access_code: Vec<u8> = buffer
                    .by_ref()
                    .take_while(|c| {*c != b' '})
                    .collect();
                let unix_access_code = str::from_utf8(&unix_access_code)?;

                // break when iterator starts yielding nothing
                if unix_access_code.is_empty() {
                    break;
                }

                let file_name: Vec<u8> = buffer
                    .by_ref()
                    .take_while(|c| {*c != b'\0'})
                    .collect();
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
        },
        _ => bail!("unknown content type")
    }
}

pub fn init_git_dir() {
    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::write(".git/HEAD", "ref: refs/heads/master\n").unwrap();
    println!("Initialized git directory")
}
