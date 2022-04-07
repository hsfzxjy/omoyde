use std::hash::{Hash, Hasher};
use std::lazy::SyncOnceCell;

use crate::prelude::*;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, PartialOrd, Ord)]
pub struct CanonicalizedPath(Arc<Path>);

impl CanonicalizedPath {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self(path.as_ref().canonicalize().unwrap().into())
    }
}

impl fmt::Display for CanonicalizedPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

pub trait Canonicalize {
    fn resolve(self) -> CanonicalizedPath;
}

impl<P: AsRef<Path>> Canonicalize for P {
    fn resolve(self) -> CanonicalizedPath {
        CanonicalizedPath::new(self)
    }
}

impl Deref for CanonicalizedPath {
    type Target = Arc<Path>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone)]
pub struct DirectoryLocation {
    pub mpid: Uuid,
    pub path: CanonicalizedPath,
}

impl DirectoryLocation {
    pub fn from_mpid(mpid: Uuid) -> Option<Self> {
        mpt_access().query(mpid).map(Self::from)
    }

    pub fn from_path(path: &CanonicalizedPath) -> Option<Self> {
        mpt_access().query(path.clone()).map(Self::from)
    }
    fn from_path_unchecked<P: AsRef<Path>>(path: P) -> Option<Self> {
        let path = CanonicalizedPath(path.as_ref().into());
        Self::from_path(&path)
    }
}

impl From<&MountPointRecord> for DirectoryLocation {
    fn from(mp: &MountPointRecord) -> Self {
        Self {
            mpid: mp.uuid,
            path: mp.path.clone(),
        }
    }
}

impl DirectoryLocation {
    pub fn with_filename<P: AsRef<Path>>(&self, filename: P) -> FileLocation {
        FileLocation {
            mpid: self.mpid,
            filename: filename.as_ref().into(),
            fullpath_cache: self.path.join(filename.as_ref()).into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileLocation {
    pub mpid: Uuid,
    pub filename: Arc<Path>,
    #[serde(skip)]
    fullpath_cache: SyncOnceCell<PathBuf>,
}

impl FileLocation {
    fn get_parts(&self) -> (Uuid, Arc<Path>) {
        (self.mpid, self.filename.clone())
    }
    pub fn filepath(&self) -> &Path {
        self.fullpath_cache.get_or_init(|| {
            DirectoryLocation::from_mpid(self.mpid)
                .unwrap()
                .path
                .join(&self.filename)
        })
    }
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().canonicalize()?;
        let dir = path
            .parent()
            .ok_or_else(|| anyhow!("cannot get parent for {}", path.display()))?;
        let filename = path
            .file_name()
            .ok_or_else(|| anyhow!("cannot get file name for {}", path.display()))?;
        let this = DirectoryLocation::from_path_unchecked(&dir)
            .ok_or_else(|| anyhow!("{} not mounted", dir.display()))?
            .with_filename(filename);
        Ok(this)
    }
}

impl PartialEq<Self> for FileLocation {
    fn eq(&self, other: &Self) -> bool {
        self.get_parts().eq(&other.get_parts())
    }
    fn ne(&self, other: &Self) -> bool {
        self.get_parts().ne(&other.get_parts())
    }
}

impl Eq for FileLocation {}

impl Hash for FileLocation {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get_parts().hash(state);
    }
}
