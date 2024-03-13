use std::{fs, os::unix::fs::MetadataExt, path::PathBuf};

use anyhow::Context;

use crate::object::{Kind, Object};

pub(crate) fn invoke(write: bool, file: &PathBuf) -> anyhow::Result<()> {
    let file = fs::File::open(file).context("opening file")?;
    let meta = file.metadata().context("getting file metadata")?;
    let size = meta.size();

    let mut object = Object {
        kind: Kind::Blob,
        size: size.try_into().context("failed to convert u64 to usize")?,
        content: file,
    };

    let hash = if write {
        object.write_to_objects()?
    } else {
        // Use io::sink() to discard the output and only get the hash
        object.write(std::io::sink())?
    };

    println!("{}", hex::encode(hash));

    Ok(())
}
