use std::io::{BufRead, Cursor, Read};

use anyhow::Context;

use crate::object::{Kind, Object};

pub(crate) fn invoke(tree_hash: &str) -> anyhow::Result<()> {
    let object = Object::read(tree_hash).context("failed to read tree object")?;

    if object.kind != Kind::Tree {
        anyhow::bail!("not a tree object: {}", tree_hash);
    }

    let mut reader = Cursor::new(object.content);
    let mut buf = Vec::new();
    let mut hashbuf = [0u8; 20];

    loop {
        buf.clear();
        let n = reader
            .read_until(b'\0', &mut buf)
            .context("failed to read header")?;

        if n == 0 {
            break;
        }

        reader
            .read_exact(&mut hashbuf)
            .context("failed to read object hash")?;

        let mut header = buf.splitn(2, |&b| b == b' ');
        let mode = std::str::from_utf8(header.next().context("failed to read mode")?)?;
        let name = std::str::from_utf8(header.next().context("failed to read name")?)?;
        let hash = hex::encode(hashbuf);

        // TODO: Optimize this
        let o = Object::read(&hash).context("failed to read object")?;

        println!("{mode:0<6} {} {hash} {name}", o.kind);
    }

    Ok(())
}
