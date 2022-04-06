use super::records::*;
use super::index::*;
use crate::prelude::*;
use crate::util::sync::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct PhotoTable {
    pub(super) counter: AtomicCounter,
    pub(super) pid2rec: BTreeMap<u32, PhotoRecord>,
    #[serde(skip)]
    pub(super) index: PhotoTableIndex,
}

impl Table for PhotoTable {
    type Record = PhotoRecord;
    type PrimaryKey = PID;

    fn insert(&mut self, mut rec: PhotoRecord) -> &mut PhotoRecord {
        let pid = self.counter.get_and_incr();
        rec.pid = pid;
        self.index.build_for(&rec);
        self.pid2rec.entry(pid).or_insert(rec)
    }
    fn remove(&mut self, slot: Occupied<'_, Self>) -> PhotoRecord {
        self.remove_index(slot.get());
        slot.remove()
    }
    fn remove_index(&mut self, rec: &Self::Record) {
        self.index.drop_for(rec);
    }
    fn treemap(&mut self) -> &mut BTreeMap<PID, PhotoRecord> {
        &mut self.pid2rec
    }
    fn after_init(&mut self) {
        self.build_index();
    }
}

impl PhotoTable {
    pub fn new() -> Self {
        Self {
            counter: AtomicCounter::new(0),
            pid2rec: BTreeMap::new(),
            index: PhotoTableIndex::default(),
        }
    }
}

impl PhotoTable {
    pub(super) fn build_index(&mut self) {
        for rec in self.pid2rec.values() {
            self.index.build_for(rec)
        }
    }
}

lazy_static! {
    pub static ref PHOTO_TABLE: Mutex<PhotoTable> = Mutex::new(PhotoTable::new());
}
