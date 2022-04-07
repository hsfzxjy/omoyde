use crate::prelude::*;
use crate::util::sync::AtomicFlag;

lazy_static! {
    pub static ref MOUNTPOINT_TABLE: Mutex<MountPointTable> = Mutex::new(MountPointTable::new());
}

impl Table for MountPointTable {
    type PrimaryKey = CanonicalizedPath;
    type Record = MountPointRecord;
    type Index = MountPointTableIndex;

    fn index_mut(&mut self) -> &mut MountPointTableIndex {
        &mut self.index
    }

    fn treemap_mut(&mut self) -> &mut BTreeMap<Self::PrimaryKey, Self::Record> {
        &mut self.path2rec
    }

    fn modified_flag(&self) -> &AtomicFlag {
        &self.modified
    }
}

#[derive(Serialize, Deserialize)]
pub struct MountPointTable {
    #[serde(skip)]
    modified: AtomicFlag,
    pub(super) path2rec: BTreeMap<CanonicalizedPath, MountPointRecord>,
    #[serde(skip)]
    pub(super) index: MountPointTableIndex,
}

impl MountPointTable {
    pub(super) fn new() -> MountPointTable {
        MountPointTable {
            modified: AtomicFlag::new(),
            index: Default::default(),
            path2rec: BTreeMap::new(),
        }
    }
}
