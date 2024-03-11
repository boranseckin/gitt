use std::fmt::Display;
use std::io::{BufWriter, Read, Write};
use std::{fs, str::FromStr};

use anyhow::Context;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};

#[derive(Debug)]
pub(crate) enum Kind {
    Blob,
    Tree,
    Commit,
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Blob => write!(f, "blob"),
            Self::Tree => write!(f, "tree"),
            Self::Commit => write!(f, "commit"),
        }
    }
}

impl FromStr for Kind {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blob" => Ok(Self::Blob),
            "tree" => Ok(Self::Tree),
            "commit" => Ok(Self::Commit),
            _ => Err(anyhow::anyhow!("unknown object kind: {}", s)),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Object {
    pub(crate) kind: Kind,
    pub(crate) size: usize,
    pub(crate) content: String,
}

impl Object {
    pub(crate) fn read(hash: &str) -> anyhow::Result<Self> {
        let object_path = format!("./.git/objects/{}/{}", &hash[..2], &hash[2..]);
        // TODO: Implement reader
        let object = fs::read(object_path).context("failed to read the object")?;

        let mut decoder = ZlibDecoder::new(object.as_slice());
        let mut buffer = String::new();
        decoder
            .read_to_string(&mut buffer)
            .context("failed to decompress the object")?;

        let (header, content) = buffer
            .split_once('\0')
            .context("failed to split the header")?;

        let (kind, size) = header
            .split_once(' ')
            .context("failed to parse the header")?;
        let kind = Kind::from_str(kind).context("failed to parse the kind")?;
        let size: usize = size.parse().context("failed to parse the size")?;

        Ok(Self {
            kind,
            size,
            content: content.to_string(),
        })
    }

    pub(crate) fn write(&self, writer: impl Write) -> anyhow::Result<String> {
        let mut object: Vec<u8> = Vec::new();
        write!(object, "{} {}\0{}", self.kind, self.size, self.content)
            .context("failed to write the object")?;
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(&object)
            .context("failed to compress object")?;
        let compressed = encoder.finish().context("failed to finish compression")?;

        let mut writer = BufWriter::new(writer);
        writer
            .write_all(&compressed)
            .context("failed to write the object")?;

        let mut hasher = Sha1::new();
        // Hash is computed over the uncompressed content including the header
        hasher.update(&object);
        let hash = hasher.finalize();

        Ok(hex::encode(hash))
    }
}
