use std::{fs, path::PathBuf};

use anyhow::Context;

use crate::object::{Kind, Object};

pub(crate) fn invoke(write: bool, file: &PathBuf) -> anyhow::Result<()> {
    // Read the file into memory as bytes
    let file = fs::read(file).context("reading file")?;

    let object = Object {
        kind: Kind::Blob,
        size: file.len(),
        content: file,
    };

    let hash = if write {
        // Write the object into a temporary file to compute its hash
        let writer = fs::File::create("temporary").context("failed to crate a temporary file")?;
        let hash = object.write(writer).context("failed to write object")?;

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
        object.write(std::io::sink())?
    };

    println!("{hash}");

    Ok(())
}
