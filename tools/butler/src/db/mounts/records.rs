use crate::prelude::*;
use crate::util::tabled::*;

use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize, Tabled)]
pub struct MountPointRecord {
    #[tabled(rename = "MPID")]
    pub uuid: Uuid,
    #[tabled(rename = "PATH")]
    pub path: CanonicalizedPath,
    #[tabled(rename = "ALIAS", display_with = "display_option")]
    pub alias: Option<String>,
}

impl TableRecord for MountPointRecord {
    type Table = MountPointTable;

    fn primary_key(&self) -> &CanonicalizedPath {
        &self.path
    }
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
