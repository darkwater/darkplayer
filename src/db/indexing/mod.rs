use std::{collections::BTreeSet, io, path::PathBuf};

use super::Ed2kHash;
use crate::message::Message;

pub fn index(path: PathBuf, existing: BTreeSet<PathBuf>) {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(
            std::thread::available_parallelism()
                .map(|n| n.get() / 2)
                .unwrap_or(1)
                .max(1),
        )
        .build()
        .unwrap();

    if let Err(e) = pool.install(|| index_inner(path, &existing)) {
        eprintln!("Error indexing path: {e}");
    }
}

fn index_inner(path: PathBuf, existing: &BTreeSet<PathBuf>) -> io::Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if let Err(e) = index_inner(path, existing) {
                eprintln!("Error indexing path: {e}");
            }
        }
    } else if path.is_file() {
        if existing.contains(&path) {
            log::debug!("File already indexed: {:?}", path);
            return Ok(());
        }

        if let Err(e) = index_file(path) {
            eprintln!("Error indexing file: {e}");
        }
    }

    Ok(())
}

fn index_file(path: PathBuf) -> ranidb::RadResult<()> {
    let hash = ranidb::ed2k::hash_file(&path, |_| {})?;

    Message::mutate_state(|state| {
        state.db.index.insert(path, Ed2kHash(hash));
    })
    .send();

    Ok(())
}
