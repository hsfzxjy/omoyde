use super::index::*;
use super::records::*;
use crate::prelude::*;
use crate::util::sync::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct PhotoTable {
    #[serde(skip)]
    modified: AtomicFlag,
    pub(super) counter: AtomicCounter,
    pub(super) pid2rec: BTreeMap<u32, PhotoRecord>,
    #[serde(skip)]
    pub(super) index: PhotoTableIndex,
}

impl Table for PhotoTable {
    type Record = PhotoRecord;
    type PrimaryKey = PID;
    type Index = PhotoTableIndex;

    fn insert(&mut self, mut rec: PhotoRecord) -> &mut PhotoRecord {
        let pid = self.counter.get_and_incr();
        rec.pid = pid;
        self.index.insert(&rec);
        self.pid2rec.entry(pid).or_insert(rec)
    }

    fn index_mut(&mut self) -> &mut PhotoTableIndex {
        &mut self.index
    }

    fn treemap_mut(&mut self) -> &mut BTreeMap<PID, PhotoRecord> {
        &mut self.pid2rec
    }

    fn modified_flag(&self) -> &AtomicFlag {
        &self.modified
    }
}

impl PhotoTable {
    pub fn new() -> Self {
        Self {
            modified: AtomicFlag::new(),
            counter: AtomicCounter::new(0),
            pid2rec: BTreeMap::new(),
            index: PhotoTableIndex::default(),
        }
    }
}

lazy_static! {
    pub static ref PHOTO_TABLE: Mutex<PhotoTable> = Mutex::new(PhotoTable::new());
}
