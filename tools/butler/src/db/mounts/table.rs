use crate::prelude::*;

lazy_static! {
    pub static ref MOUNTPOINT_TABLE: Mutex<MountPointTable> = Mutex::new(MountPointTable::new());
}

impl Table for MountPointTable {
    type PrimaryKey = CanonicalizedPath;
    type Record = MountPointRecord;

    fn insert(&mut self, rec: Self::Record) -> &mut Self::Record {
        let uuid = rec.uuid;
        let path = rec.path.clone();
        self.uuid2path.insert(uuid, path.clone());
        self.path2rec.entry(path.clone()).or_insert(rec)
    }

    fn remove(&mut self, slot: Occupied<'_, Self>) -> Self::Record {
        self.remove_index(slot.get());
        slot.remove()
    }
    fn remove_index(&mut self, rec: &Self::Record) {
        self.uuid2path.remove(&rec.uuid);
    }

    fn treemap(&mut self) -> &mut BTreeMap<Self::PrimaryKey, Self::Record> {
        &mut self.path2rec
    }

    fn after_init(&mut self) {}
}

pub struct MountPointTable {
    pub(super) path2rec: BTreeMap<CanonicalizedPath, MountPointRecord>,
    pub(super) uuid2path: HashMap<Uuid, CanonicalizedPath>,
}

impl MountPointTable {
    pub(super) fn new() -> MountPointTable {
        MountPointTable {
            uuid2path: HashMap::new(),
            path2rec: BTreeMap::new(),
        }
    }
}

impl std::fmt::Display for MountPointTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for entry in self.access().records() {
            writeln!(
                f,
                "{} {} {}",
                entry.uuid,
                Path::new(entry.path.as_ref()).display(),
                entry.alias.as_ref().unwrap_or(&"<NOALIAS>".to_string())
            )?;
        }
        Ok(())
    }
}
