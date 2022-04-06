use crate::prelude::*;

use super::misc::*;

use crate::_vendors::filebuffer;
use xxhash_rust::xxh3::xxh3_64;

#[derive(Debug)]
pub struct LocalPhoto {
    pub location: Arc<FileLocation>,
    pub file_hash: Option<FileHash>,
    pub metadata: PhotoMetadata,
    mmap: Option<filebuffer::FileBuffer>,
}

impl LocalPhoto {
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
pub struct PhotoRecord {
    pub pid: PID, // unique ID of image

    // file path related
    pub location: Arc<FileLocation>,
    pub file_hash: FileHash,
    pub metadata: PhotoMetadata,

    // omoyde related
    pub selected: bool,
    pub status: PhotoRecordStatus,
    pub commit_time: Option<DateTime<Utc>>,
}

impl PhotoRecord {
    pub fn new(pid: PID, file: LocalPhoto) -> Self {
        Self {
            pid,
            location: file.location.into(),
            file_hash: file.file_hash.unwrap(),
            metadata: file.metadata,
            selected: false,
            status: Uncommitted,
            commit_time: None,
        }
    }
}
