use std::io::{self, Write};

use anyhow::Context;

use crate::object::{Kind, Object};

pub(crate) fn invoke(pretty_print: bool, object_hash: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        pretty_print,
        "object kind must be provided, but it's not implemented yet. Use -p option instead."
    );

    let object = Object::read(object_hash).context("failed to read object")?;

    if object.kind != Kind::Blob {
        anyhow::bail!("object type {:?} is not ye implemented", object.kind);
    }

    io::stdout()
        .write_all(&object.content)
        .context("failed to write to stdout")?;

    Ok(())
}
