use std::io::{BufRead, Read, Write};

use anyhow::Context;

use crate::object::{Kind, Object};

pub(crate) fn invoke(tree_hash: &str) -> anyhow::Result<()> {
    let mut tree_object = Object::read(tree_hash).context("failed to read tree object")?;

    if tree_object.kind != Kind::Tree {
        anyhow::bail!("not a tree object: {}", tree_hash);
    }

    /*
     * Contents of a tree object (added newlines for readability):
     *
     * tree <size>\0
     * <mode> <name>\0<20_byte_sha>
     * <mode> <name>\0<20_byte_sha>
     */

    let mut buf = Vec::new();
    let mut hashbuf = [0u8; 20];

    let mut stdout = std::io::stdout().lock();

    loop {
        buf.clear();

        // Read mode and name (including the null byte)
        let n = tree_object
            .content
            .read_until(b'\0', &mut buf)
            .context("failed to read header")?;

        // If nothing was read, we reached the EOF
        if n == 0 {
            break;
        }

        // Read 20-byte hash
        tree_object
            .content
            .read_exact(&mut hashbuf)
            .context("failed to read object hash")?;

        // The length of mode is variable, and so is the length of the name
        // We can't use splitn() here, so we'll have to manually find the space
        let mut header = buf.splitn(2, |&b| b == b' ');
        let mode = std::str::from_utf8(header.next().context("failed to read mode")?)?;
        let name = std::str::from_utf8(header.next().context("failed to read name")?)?;
        let hash = hex::encode(hashbuf);

        let object = Object::read(&hash).context("failed to read object")?;

        writeln!(stdout, "{mode:0>6} {} {hash} {name}", object.kind)
            .context("failed to write to stdout")?;
    }

    Ok(())
}
