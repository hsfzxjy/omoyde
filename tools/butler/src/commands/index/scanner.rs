use std::fs::DirEntry;

use crate::prelude::*;
use crate::util;
use crate::util::functional::*;


fn is_file(file: &std::io::Result<DirEntry>) -> bool {
    file.as_ref()
        .ok()
        .map(|f| f.file_type().ok())
        .flatten()
        .map(|t| t.is_file())
        .unwrap_or(false)
}

fn is_supported_image(file: &std::io::Result<DirEntry>) -> bool {
    file.as_ref()
        .ok()
        .map(|f| util::fs::is_supported_image(&f.path()))
        .unwrap_or(false)
}

struct ClassifiedResult<'a> {
    existing_pids: HashSet<PID>,
    new_lphotos: Vec<LocalPhoto>,
    photo_record_diffs: Vec<PhotoRecordDiff<'a>>,
}

impl<'a> ClassifiedResult<'a> {
    fn new() -> Self {
        Self {
            existing_pids: HashSet::new(),
            new_lphotos: vec![],
            photo_record_diffs: vec![],
        }
    }
    fn into_tuple(self) -> (HashSet<PID>, Vec<LocalPhoto>, Vec<PhotoRecordDiff<'a>>) {
        (
            self.existing_pids,
            self.new_lphotos,
            self.photo_record_diffs,
        )
    }
}

struct MountPointScanner<'b, 'a: 'b> {
    loc: DirectoryLocation,
    access: &'b PhotoTableAccessMut<'a>,
}

impl<'b, 'a: 'b> MountPointScanner<'b, 'a> {
    fn new(mp: &MountPointRecord, access: &'b PhotoTableAccessMut<'a>) -> Result<Self> {
        Ok(Self {
            loc: mp.into(),
            access,
        })
    }
    fn sorted_files(&self) -> Result<impl IntoIterator<Item = DirEntry>> {
        use std::os::unix::fs::DirEntryExt;
        let mut files = self
            .loc
            .path
            .read_dir()?
            .filter(is_file)
            .filter(is_supported_image)
            .collect::<StdResult<Vec<_>, _>>()?;
        files.sort_by_cached_key(DirEntry::ino);
        Ok(files)
    }
    fn to_lphoto(&self, file: DirEntry) -> Result<LocalPhoto> {
        let loc = self.loc.with_filename(file.file_name());
        let lphoto = LocalPhoto::new(loc)?;
        Ok(lphoto)
    }
    fn classify<In>(&self, lphotos: In) -> Result<ClassifiedResult<'a>>
    where
        In: IntoIterator<Item = Result<LocalPhoto>>,
    {
        let mut res = ClassifiedResult::new();
        for lphoto in lphotos.into_iter() {
            let mut lphoto = lphoto?;
            let rec = unwrap_some_or!(self.access.query::<Arc<_>, _>(&lphoto.location), {
                lphoto.prefetch()?;
                lphoto.fill_file_hash()?;
                res.new_lphotos.push(lphoto);
                continue;
            });

            let need_hash_check =
                lphoto.metadata != rec.metadata || rec.status == CommittedButMissing;
            if !need_hash_check {
                res.existing_pids.insert(rec.pid);
                continue;
            }

            lphoto.prefetch()?;
            lphoto.fill_file_hash()?;
            let mut diff = PhotoRecordDiff::new(rec);
            diff.metadata.set(lphoto.metadata.clone());
            diff.file_hash.set(lphoto.file_hash.unwrap());
            res.photo_record_diffs.push(diff);
        }
        Ok(res)
    }
    fn run(self) -> Result<ClassifiedResult<'a>> {
        let lphotos = self
            .sorted_files()?
            .into_iter()
            .map(|file| self.to_lphoto(file));
        self.classify(lphotos)
    }
}

pub struct Scanner<'b, 'a: 'b> {
    pt: &'b mut PhotoTableAccessMut<'a>,
    mpt: &'b TableAccess<'a, MountPointTable>,
}

impl<'b, 'a: 'b> Scanner<'b, 'a> {
    pub fn new(
        pt: &'b mut PhotoTableAccessMut<'a>,
        mpt: &'b TableAccess<'a, MountPointTable>,
    ) -> Self {
        Self { pt, mpt }
    }
    pub fn run(self) -> Result<()> {
        let (mut pids, new_lphotos, diffs) = self
            .mpt
            .records()
            .map(|mp| MountPointScanner::new(mp, &self.pt))
            .map(|s| -> Result<_> { Ok(s?.run()?.into_tuple()) })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .fold_into();

        new_lphotos.into_iter().for_each(|file| {
            let pid = self.pt.insert_lphoto(file);
            pids.insert(pid);
        });

        diffs.into_iter().for_each(|diff| {
            let pid = diff.pid;
            self.pt.take_diff(diff).commit();
            pids.insert(pid);
        });

        self.pt.retain(|pid, rec| {
            if pids.contains(pid) {
                return true;
            }
            rec.status_mut().handle_local_missing();
            *rec.selected()
        });

        Ok(())
    }
}
