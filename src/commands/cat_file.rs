use std::io;

use anyhow::Context;

use crate::object::{Kind, Object};

pub(crate) fn invoke(pretty_print: bool, object_hash: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        pretty_print,
        "object kind must be provided, but it's not implemented yet. Use -p option instead."
    );

    let mut object = Object::read(object_hash).context("failed to read object")?;

    match object.kind {
        Kind::Blob => {
            let n = io::copy(&mut object.content, &mut io::stdout())
                .context("failed to write to object content to stdout")?;
            anyhow::ensure!(n == object.size as u64, "object size mismatch");
        }
        _ => anyhow::bail!("object type {:?} is not yet implemented", object.kind),
    }

    Ok(())
}
