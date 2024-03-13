use std::fmt::Display;
use std::fs;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::str::FromStr;

use anyhow::Context;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};

pub(crate) type Hash = [u8; 20];

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
pub(crate) struct Object<R> {
    pub(crate) kind: Kind,
    pub(crate) size: usize,
    pub(crate) content: R,
}

impl Object<()> {
    pub(crate) fn read(hash: &str) -> anyhow::Result<Object<impl BufRead>> {
        let object_path = format!("./.git/objects/{}/{}", &hash[..2], &hash[2..]);

        let file = fs::File::open(object_path).context("failed to open the object file")?;

        let decoder = ZlibDecoder::new(file);
        let mut reader = BufReader::new(decoder);
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

        Ok(Object {
            kind,
            size,
            content: reader,
        })
    }
}

impl<R> Object<R>
where
    R: Read,
{
    pub(crate) fn write(&mut self, writer: impl Write) -> anyhow::Result<Hash> {
        let encoder = ZlibEncoder::new(writer, Compression::default());

        let mut writer = HashWriter {
            hasher: Sha1::new(),
            writer: encoder,
        };

        write!(writer, "{} {}\0", self.kind, self.size)
            .context("failed to write the object header")?;
        io::copy(&mut self.content, &mut writer)?;

        writer
            .writer
            .finish()
            .context("failed to finish compression")?;

        let hash = writer.hasher.finalize();

        Ok(hash.into())
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
