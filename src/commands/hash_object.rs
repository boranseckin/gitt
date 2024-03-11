use std::{fs, path::PathBuf};

use anyhow::Context;

use crate::object::{Kind, Object};

pub(crate) fn invoke(write: bool, file: &PathBuf) -> anyhow::Result<()> {
    let file = fs::read_to_string(file).context("reading file")?;

    let object = Object {
        kind: Kind::Blob,
        size: file.len(),
        content: file,
    };

    let hash = if write {
        let writer = fs::File::create("temporary")?;
        let hash = object.write(writer)?;

        fs::create_dir_all(format!(".git/objects/{}/", &hash[..2]))
            .context("failed to create subdir")?;
        fs::rename(
            "temporary",
            format!(".git/objects/{}/{}", &hash[..2], &hash[2..]),
        )
        .context("failed to move temporary object")?;

        hash
    } else {
        object.write(std::io::sink())?
    };

    println!("{}", hash);

    Ok(())
}
