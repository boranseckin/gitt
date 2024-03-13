use std::{fs, os::unix::fs::MetadataExt, path::PathBuf};

use anyhow::Context;

use crate::object::{Kind, Object};

pub(crate) fn invoke(write: bool, file: &PathBuf) -> anyhow::Result<()> {
    // Read the file into memory as bytes
    let file = fs::File::open(file).context("opening file")?;
    let meta = file.metadata().context("getting file metadata")?;
    let size = meta.size();

    let mut object = Object {
        kind: Kind::Blob,
        size: size as usize,
        content: file,
    };

    let hash = if write {
        // Write the object into a temporary file to compute its hash
        // TODO: use better temporary file handling
        let writer = fs::File::create("temporary").context("failed to crate a temporary file")?;
        let hash = object.write(writer).context("failed to write object")?;
        let hash = hex::encode(hash);

        // Move the temporary file to the object database
        fs::create_dir_all(format!(".git/objects/{}/", &hash[..2]))
            .context("failed to create subdir")?;
        fs::rename(
            "temporary",
            format!(".git/objects/{}/{}", &hash[..2], &hash[2..]),
        )
        .context("failed to move temporary object")?;

        hash
    } else {
        // Use io::sink() to discard the output and only get the hash
        let hash = object.write(std::io::sink())?;
        hex::encode(hash)
    };

    println!("{hash}");

    Ok(())
}
