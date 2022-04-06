use crate::prelude::*;

impl Serialize for MountPointTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.path2rec.len()))?;
        for entry in self.path2rec.values() {
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
        while let Some(entry) = access.next_element::<MountPointRecord>()? {
            ret.access_mut().insert(entry);
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
