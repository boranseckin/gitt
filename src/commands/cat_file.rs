use anyhow::Context;

use crate::object::Object;

pub(crate) fn invoke(pretty_print: bool, object_hash: &str) -> anyhow::Result<()> {
    anyhow::ensure!(
        pretty_print,
        "object kind must be provided, but it's not implemented yet"
    );

    let object = Object::read(object_hash).context("failed to read object")?;
    let hash = std::str::from_utf8(&object.content)?;

    print!("{hash}");

    Ok(())
}
