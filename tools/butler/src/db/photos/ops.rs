use super::super::mounts::*;
use super::*;
use crate::db::*;
use crate::prelude::*;
use crate::util;
use crate::util::serde::*;

pub fn initialize_mountpoint_table() -> Result<()> {
    let mut mpt = MOUNTPOINT_TABLE.lock().unwrap();
    mpt.load_from_path(".butler/mountpoints")?;
    Ok(())
}
pub fn finalize_mountpoint_table() -> Result<()> {
    let mpt = MOUNTPOINT_TABLE.lock().unwrap();
    mpt.save_to_path(".butler/mountpoints")?;
    Ok(())
}

pub fn scan_photos(
) -> Result<impl IntoIterator<Item = impl IntoIterator<Item = Result<Arc<LocalPhotoEntry>>>>> {
    use std::os::unix::fs::DirEntryExt;

    let mpt = MOUNTPOINT_TABLE.lock().unwrap();

    let entries = mpt
        .path2entry
        .iter()
        .map(|(path, mp)| -> Result<_> {
            let loc = DirectoryLocation::from(mp);
            let ret = path.read_dir()?;
            Ok((loc, ret))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .map(|(loc, read_dir)| {
            let mut res = read_dir
                .filter_map(Result::ok)
                .filter(|file| file.file_type().map_or(false, |t| t.is_file()))
                .filter(|file| util::fs::is_supported_image(&file.path()))
                .map(move |file| -> Result<_> {
                    let loc = loc.with_filename(file.file_name());
                    let entry = LocalPhotoEntry::new(loc)?;
                    Ok((file.ino(), Arc::new(entry)))
                })
                .filter_map(Result::ok)
                .collect::<Vec<_>>();
            res.sort_by(|a, b| a.0.cmp(&b.0));
            res.into_iter().map(|(_, entry)| Ok(entry))
        });

    println!("Scanned {} local entries.", entries.len());
    Ok(entries)
}
