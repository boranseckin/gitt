use std::fmt::Display;
use std::io::{BufRead, Cursor, Read, Write};
use std::{fs, str::FromStr};

use anyhow::Context;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};

#[derive(Debug, PartialEq, Eq)]
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
    pub(crate) content: Vec<u8>,
}

impl Object {
    pub(crate) fn read(hash: &str) -> anyhow::Result<Self> {
        let object_path = format!("./.git/objects/{}/{}", &hash[..2], &hash[2..]);
        // TODO: Implement reader
        let object = fs::read(object_path).context("failed to read the object")?;

        let mut decoder = ZlibDecoder::new(object.as_slice());
        let mut buffer = Vec::new();
        decoder
            .read_to_end(&mut buffer)
            .context("failed to decompress the object")?;

        // Split the header from the content by finding the first null byte
        let mut reader = Cursor::new(buffer);
        let mut header = Vec::new();
        reader
            .read_until(b'\0', &mut header)
            .context("failed to read the header")?;
        header.pop(); // Remove the null byte
        let header = std::str::from_utf8(&header).context("failed to parse the header")?;

        let (kind, size) = header
            .split_once(' ')
            .context("failed to parse the header")?;

        let kind = Kind::from_str(kind).context("failed to parse the kind")?;
        let size: usize = size.parse().context("failed to parse the size")?;

        let mut content = Vec::new();
        reader
            .read_to_end(&mut content)
            .context("failed to read the content")?;

        Ok(Self {
            kind,
            size,
            content,
        })
    }

    pub(crate) fn write(&self, writer: impl Write) -> anyhow::Result<String> {
        let encoder = ZlibEncoder::new(writer, Compression::default());

        let mut writer = HashWriter {
            hasher: Sha1::new(),
            writer: encoder,
        };

        write!(writer, "{} {}\0", self.kind, self.size)
            .context("failed to write the object header")?;
        writer
            .write_all(&self.content)
            .context("failed to write the object content")?;

        writer
            .writer
            .finish()
            .context("failed to finish compression")?;

        let hash = writer.hasher.finalize();

        Ok(hex::encode(hash))
    }
}

/// A wrapper around a writer that computes the SHA-1 hash of the written data
/// and writes the data to the underlying writer.
struct HashWriter<W> {
    hasher: Sha1,
    writer: W,
}

impl<W: Write> Write for HashWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
