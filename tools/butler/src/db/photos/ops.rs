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

pub fn scan_photos() -> Result<impl IntoIterator<Item = Arc<LocalPhotoEntry>>> {
    use rayon::prelude::*;
    let mpt = MOUNTPOINT_TABLE.lock().unwrap();

    let entries = mpt
        .path2entry
        .iter()
        .map(|(path, mp)| -> Result<_> {
            let loc = Arc::new(DirectoryLocation::from(mp));
            let ret = path.read_dir()?;
            Ok((loc, ret))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flat_map(|(loc, read_dir)| {
            read_dir.map(move |file| -> Result<_> { Ok((loc.clone(), file?)) })
        })
        .collect::<Result<Vec<_>>>()?
        .into_par_iter()
        .map(|(loc, file)| -> Result<_> {
            if file.file_type()?.is_file() && util::fs::is_supported_image(&file.path()) {
                let loc = loc.with_filename(file.file_name());
                let entry = LocalPhotoEntry::new(loc)?;
                Ok(Some(Arc::new(entry)))
            } else {
                Ok(None)
            }
        })
        .filter_map(Result::transpose)
        .collect::<Result<Vec<_>>>()?;

    println!("Scanned {} local entries.", entries.len());
    Ok(entries)
}
