use crate::prelude::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountPointRecord {
    pub uuid: Uuid,
    pub alias: Option<String>,
    pub path: CanonicalizedPath,
}

impl MountPointRecord {
    pub(super) fn new(path: CanonicalizedPath, alias: Option<String>) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            path,
            alias,
        }
    }
}
