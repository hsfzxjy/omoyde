use crate::prelude::*;

use super::misc::*;
use crate::db::locations::*;

#[derive(Debug)]
pub struct LocalPhotoEntry {
    pub location: Arc<FileLocation>,
    pub file_hash: Option<FileHash>,
    pub metadata: PhotoMetadata,
}

impl LocalPhotoEntry {
    pub fn new(location: FileLocation) -> Result<Self> {
        let metadata = PhotoMetadata::from_path(location.filepath())?;
        Ok(Self {
            location: location.into(),
            metadata,
            file_hash: None,
        })
    }
    fn filepath(&self) -> &Path {
        self.location.filepath()
    }
    pub fn fill_file_hash(&mut self) -> Result<()> {
        println!("Hashing {}...", self.filepath().display());
        if self.file_hash.is_some() {
            return Ok(());
        }
        let mut reader = File::open(self.filepath())?;
        let mut hasher = fasthash::xx::Hasher64::new();
        hasher.write_stream(&mut reader)?;
        let hash_val = hasher.finish();
        self.file_hash = Some(hash_val);
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
