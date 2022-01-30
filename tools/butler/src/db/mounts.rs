use super::locations::*;
use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountPointEntry {
    pub uuid: Uuid,
    pub alias: Option<String>,
    pub path: CanonicalizedPath,
}

impl MountPointEntry {
    fn from_path_and_alias(path: CanonicalizedPath, alias: Option<String>) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            path: path,
            alias,
        }
    }
}

pub struct MountPointTable {
    pub path2entry: BTreeMap<CanonicalizedPath, MountPointEntry>,
    pub uuid2path: HashMap<Uuid, CanonicalizedPath>,
}

impl MountPointTable {
    fn add_entry(&mut self, entry: MountPointEntry) {
        let uuid = entry.uuid;
        let path = entry.path.clone();
        self.path2entry.insert(path.clone(), entry);
        self.uuid2path.insert(uuid, path);
    }
}
impl MountPointTable {
    pub fn get_by_mpid(&self, mpid: Uuid) -> Option<&MountPointEntry> {
        self.uuid2path
            .get(&mpid)
            .and_then(|path| self.get_by_path(path))
    }
    pub fn get_by_path(&self, path: &CanonicalizedPath) -> Option<&MountPointEntry> {
        self.path2entry.get(path)
    }
    pub fn get_mut_by_path(&mut self, path: &CanonicalizedPath) -> Option<&mut MountPointEntry> {
        self.path2entry.get_mut(path)
    }
}
impl MountPointTable {
    pub fn remove(&mut self, mpid: Uuid) -> bool {
        self.get_by_mpid(mpid)
            .map(|entry| (entry.path.clone()))
            .map(|path| {
                self.uuid2path.remove(&mpid);
                self.path2entry.remove(&path);
            })
            .is_some()
    }
    pub fn add_or_modify(&mut self, path: CanonicalizedPath, alias: Option<String>) {
        match self.get_mut_by_path(&path) {
            Some(entry) => {
                if alias.is_some() {
                    entry.alias = alias
                }
            }
            _ => {
                let entry = MountPointEntry::from_path_and_alias(path, alias);
                self.add_entry(entry);
            }
        }
    }
    fn new() -> MountPointTable {
        MountPointTable {
            uuid2path: HashMap::new(),
            path2entry: BTreeMap::new(),
        }
    }
}

impl Serialize for MountPointTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.path2entry.len()))?;
        for entry in self.path2entry.values() {
            seq.serialize_element(entry)?;
        }
        seq.end()
    }
}

struct MountPointTableVisitor;

impl<'de> serde::de::Visitor<'de> for MountPointTableVisitor {
    type Value = MountPointTable;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("table")
    }
    fn visit_seq<M>(self, mut access: M) -> Result<Self::Value, M::Error>
    where
        M: serde::de::SeqAccess<'de>,
    {
        let mut ret = Self::Value::new();
        while let Some(entry) = access.next_element::<MountPointEntry>()? {
            ret.add_entry(entry);
        }
        Ok(ret)
    }
}

impl<'de> Deserialize<'de> for MountPointTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(MountPointTableVisitor)
    }
}

impl std::fmt::Display for MountPointTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for entry in self.path2entry.values() {
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

lazy_static! {
    pub static ref MOUNTPOINT_TABLE: Mutex<MountPointTable> = Mutex::new(MountPointTable::new());
}
