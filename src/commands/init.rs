use std::{fs, path::Path};

pub(crate) fn invoke(force: bool) -> anyhow::Result<()> {
    if Path::new(".git").exists() {
        if force {
            println!("Forcing the creation of a new Git repository.");
        } else {
            anyhow::bail!("Git repository already exists in this directory.");
        }
    }

    fs::create_dir(".git").unwrap();
    fs::create_dir(".git/objects").unwrap();
    fs::create_dir(".git/refs").unwrap();
    fs::create_dir(".git/refs/tags").unwrap();
    fs::create_dir(".git/refs/heads").unwrap();

    fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();

    println!("Initialized empty Git repository in .git");

    Ok(())
}
