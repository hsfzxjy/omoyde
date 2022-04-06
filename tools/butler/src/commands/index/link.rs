use crate::prelude::*;

pub(super) fn symlink_all_photos_to<P>(pt: &TableAccessMut<'_, PhotoTable>, dir: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir)?;
    for rec in pt.records() {
        let dst = PathBuf::new().join(&dir).join(format!(
            "{}.{}",
            rec.pid,
            rec.location.filename.extension().unwrap().to_string_lossy()
        ));
        std::os::unix::fs::symlink(rec.location.filepath(), dst.as_path())?;
    }
    Ok(())
}
