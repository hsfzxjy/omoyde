use crate::prelude::*;

use super::misc::*;
use crate::db::locations::*;

use crate::_vendors::filebuffer;
use xxhash_rust::xxh3::xxh3_64;

#[derive(Debug)]
pub struct LocalPhotoEntry {
    pub location: Arc<FileLocation>,
    pub file_hash: Option<FileHash>,
    pub metadata: PhotoMetadata,
    mmap: Option<filebuffer::FileBuffer>,
}

impl LocalPhotoEntry {
    pub fn new(location: FileLocation) -> Result<Self> {
        let metadata = PhotoMetadata::from_path(location.filepath())?;
        Ok(Self {
            location: location.into(),
            metadata,
            file_hash: None,
            mmap: None,
        })
    }
    fn filepath(&self) -> &Path {
        self.location.filepath()
    }
    pub fn prefetch(&mut self) -> Result<()> {
        let mmap = filebuffer::FileBuffer::open(self.filepath())?;
        mmap.prefetch(0, mmap.len());
        self.mmap.replace(mmap);
        Ok(())
    }
    pub fn fill_file_hash(&mut self) -> Result<()> {
        if self.file_hash.is_some() {
            return Ok(());
        }
        println!("Hashing {}...", self.filepath().display());
        let mmap = self.mmap.take().unwrap();
        let buf = mmap.as_ref();
        self.file_hash = Some(xxh3_64(buf));
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DBPhotoEntry {
    pub pid: PID, // unique ID of image

    // file path related
    pub location: Arc<FileLocation>,
    pub file_hash: FileHash,
    pub metadata: PhotoMetadata,

    // omoyde related
    pub selected: bool,
    pub status: PhotoEntryStatus,
    pub commit_time: Option<DateTime<Utc>>,
}

impl DBPhotoEntry {
    pub fn new(pid: PID, local_entry: LocalPhotoEntry) -> Self {
        Self {
            pid,
            location: local_entry.location.into(),
            file_hash: local_entry.file_hash.unwrap(),
            metadata: local_entry.metadata,
            selected: false,
            status: PhotoEntryStatus::Uncommitted,
            commit_time: None,
        }
    }
    pub fn fix_metadata_from(&mut self, metadata: PhotoMetadata) {
        if self.metadata.etime.is_some() {
            println!("already fixed");
            return;
        }
        self.metadata.etime = metadata.etime;
    }
    pub fn handle_local_missing(&mut self) -> bool {
        use PhotoEntryStatus::*;
        match self.status {
            Uncommitted => false,
            _ => {
                self.status = CommittedButMissing;
                true
            }
        }
    }
}
