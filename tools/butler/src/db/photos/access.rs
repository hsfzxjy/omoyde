use crate::prelude::*;

#[allow(dead_code)]
pub fn pt_access<'a>() -> TableAccess<'a, PhotoTable> {
    PHOTO_TABLE.lock().unwrap().access()
}

pub fn pt_access_mut<'a>() -> TableAccessMut<'a, PhotoTable> {
    PHOTO_TABLE.lock().unwrap().access_mut()
}

#[allow(dead_code)]
pub type PhotoTableAccess<'a> = TableAccess<'a, PhotoTable>;
pub type PhotoTableAccessMut<'a> = TableAccessMut<'a, PhotoTable>;

impl<'b, 'a: 'b> TableEntryTrait<'b, 'a, PhotoTable> for TableEntry<'b, 'a, PhotoTable> {
    type Patch = PhotoRecordPatch<'b, 'a>;
}

impl<'a> TableAccess<'a, PhotoTable> {
    pub fn summary(&self) {
        let ptr = unsafe { self.0.as_mut() };
        let ntotal = ptr.pid2rec.len();
        let nunc = ptr.index.count_status(Uncommitted);
        let ncsel = ptr.index.count_selected(Committed);
        let ndsel = ptr.index.count_selected(CommittedButMissing);
        let nmsel = ptr.index.count_selected(CommittedButModified);
        println!(
            "Among {ntotal} photos, {nc} reviewed.",
            ntotal = ntotal,
            nc = ntotal - nunc
        );
        println!(
            "Among {nc} reviewed photos, \
            {ncsel} selected, \
            {ndsel} selected but missing, \
            {nmsel} selected but modified.",
            nc = ntotal - nunc,
            ncsel = ncsel,
            ndsel = ndsel,
            nmsel = nmsel
        );
    }
    pub fn display_list(&self, mpid: Option<Uuid>) -> Result<()> {
        for rec in self.records() {
            if let Some(uuid) = mpid {
                if rec.location.mpid != uuid {
                    continue;
                }
            }
            let mpid = rec.location.mpid;
            let fname = &rec.location.as_ref().filename;
            let exif_time = rec
                .metadata
                .etime
                .map(|t| t.timestamp().to_string())
                .unwrap_or_else(|| "none".to_string());
            let ctime = rec.metadata.ctime.timestamp().to_string();
            println!(
                "{} {} {} {:?} {} {} {} {}",
                rec.pid,
                mpid,
                fname.display(),
                rec.status,
                rec.selected,
                exif_time,
                ctime,
                rec.commit_time
                    .map(|time| time.timestamp().to_string())
                    .unwrap_or("none".to_string())
            );
        }
        Ok(())
    }
}

impl<'a> TableAccessMut<'a, PhotoTable> {
    pub fn take_diff<'b, 'c>(&'b mut self, diff: PhotoRecordDiff<'c>) -> PhotoRecordPatch<'b, 'a>
    where
        'a: 'b,
        'c: 'a,
    {
        self.entry(diff.pid).modify().unwrap().with_diff(diff)
    }
    pub fn insert_lphoto(&mut self, file: LocalPhoto) -> PID {
        let rec = PhotoRecord::new(0, file);
        self.insert_and_view(rec).pid
    }
}
