extern crate bincode;
extern crate clap_port_flag;
extern crate deflate;
extern crate fst;
extern crate futures;
extern crate hyper;
extern crate memmap;
extern crate mime_guess;
extern crate quicli;
extern crate tokio;
extern crate walkdir;

use std::fs::File;
use std::path::Path;
use std::result::Result;

use quicli::prelude::*;
use walkdir::WalkDir;

mod slice;

pub use server::serve;
mod server;

mod site;
pub use site::Site;
use walkdir::DirEntry;

pub fn build(src: &Path, target: &Path) -> Result<(), Error> {
    info!(
        "trying to build an index and archive from `{}`",
        src.display()
    );
    use std::io::{BufWriter, Write};
    let src = Box::new(src.to_path_buf());
    let src = &*Box::leak(src);

    ensure!(src.is_dir(), "Directory `{}` doesn't exist", src.display());

    let index_path = target.with_extension("index");
    let index = BufWriter::new(
        File::create(&index_path)
            .with_context(|e| format!("couldn't create file `{}`: {}", target.display(), e))?,
    );
    let index = fst::MapBuilder::new(index)
        .with_context(|e| format!("couldn't create index file `{}`: {}", target.display(), e))?;
    info!("will write index to `{}`", index_path.display());

    let archive_path = target.with_extension("archive");
    let mut archive = BufWriter::new(
        File::create(&archive_path)
            .with_context(|e| format!("couldn't create file `{}`: {}", target.display(), e))?,
    );
    info!("will write archive to `{}`", archive_path.display());

    let mut archive_index = 0;

    fn rel_as_bytes<'a>(p: &'a DirEntry, src: &Path) -> Vec<u8> {
        p.path()
            .strip_prefix(src)
            .unwrap()
            .to_string_lossy()
            .to_string()
            .into_bytes()
    }

    WalkDir::new(src)
        .sort_by(move |a, b| rel_as_bytes(a, src).cmp(&rel_as_bytes(b, src)))
        .contents_first(true)
        .into_iter()
        .flat_map(|entry| entry.map_err(|e| warn!("Couldn't read dir entry {}", e)))
        .filter(|f| f.path().is_file())
        .try_fold(index, |mut map, file| -> Result<_, Error> {
            let path = file.path();
            trace!("trying to add {} to index", path.display());
            let file_content = get_compressed_content(&path).with_context(|_| {
                format!("Could not read/compress content of {}", path.display())
            })?;
            archive.write_all(&file_content).with_context(|_| {
                format!(
                    "Could not write compressed content to {}",
                    archive_path.display()
                )
            })?;

            let rel_path = file
                .path()
                .strip_prefix(src)
                .with_context(|_| format!("Couldn't get relative path for `{:?}`", path.display()))?
                .to_path_buf();

            map.insert(
                rel_path.to_string_lossy().as_bytes(),
                slice::pack_in_u64(archive_index, file_content.len()),
            ).with_context(|_| format!("Could not insert file {} into index", path.display()))?;
            archive_index += file_content.len();
            Ok(map)
        })?.finish()
        .with_context(|e| format!("Could not finish building index: {}", e))?;

    Ok(())
}

fn get_compressed_content(path: &Path) -> Result<Vec<u8>, Error> {
    use std::fs::read;
    use std::io::Write;

    use deflate::write::GzEncoder;
    use deflate::Compression;

    let data =
        read(path).with_context(|e| format!("Couldn't read file {}: {}", path.display(), e))?;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::Best);
    encoder.write_all(&data)?;
    let compressed_data = encoder.finish()?;

    Ok(compressed_data)
}
