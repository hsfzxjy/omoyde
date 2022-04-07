use crate::prelude::*;

#[derive(Default)]
pub struct MountPointTableIndex {
    pub(super) uuid2path: HashMap<Uuid, CanonicalizedPath>,
}

impl TableIndex for MountPointTableIndex {
    type Table = MountPointTable;

    fn remove(&mut self, rec: &MountPointRecord) {
        self.uuid2path.remove(&rec.uuid);
    }

    fn insert(&mut self, rec: &MountPointRecord) {
        self.uuid2path.insert(rec.uuid, rec.path.clone());
    }
}
