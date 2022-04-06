use crate::db::*;
use crate::prelude::*;

impl TableKey<PhotoTable> for Arc<FileLocation> {
    fn query_in<'a, 'b>(&'a self, table: &'b mut PhotoTable) -> TableHandle<'b, PhotoTable> {
        table
            .index
            .loc2pid
            .get(self)
            .cloned()
            .map(|pid| pid.query_in(table))
            .into()
    }
}

impl TableKey<PhotoTable> for PID {
    fn query_in<'a, 'b>(&'a self, table: &'b mut PhotoTable) -> TableHandle<'b, PhotoTable> {
        Some(table.pid2rec.entry(self.clone())).into()
    }
}

#[derive(Clone, Debug)]
pub enum PhotoQuery {
    PID(PID),
    FileLocation(Arc<FileLocation>),
}

impl TableKey<PhotoTable> for PhotoQuery {
    fn query_in<'a, 'b>(&'a self, table: &'b mut PhotoTable) -> TableHandle<'b, PhotoTable> {
        use PhotoQuery::*;
        match self {
            PID(x) => x.query_in(table),
            FileLocation(x) => x.query_in(table),
        }
    }
}

impl From<Arc<FileLocation>> for PhotoQuery {
    fn from(v: Arc<FileLocation>) -> Self {
        PhotoQuery::FileLocation(v)
    }
}

impl From<FileLocation> for PhotoQuery {
    fn from(v: FileLocation) -> Self {
        PhotoQuery::FileLocation(v.into())
    }
}

impl From<PID> for PhotoQuery {
    fn from(v: PID) -> Self {
        PhotoQuery::PID(v)
    }
}

impl<T> From<T> for PhotoQuery
where
    T: Borrow<LocalPhoto>,
{
    fn from(v: T) -> Self {
        v.borrow().location.clone().into()
    }
}

impl std::str::FromStr for PhotoQuery {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(v) = u32::from_str(s).ok() {
            return Ok(v.into());
        }
        if let Some(v) = PathBuf::from_str(s).ok() {
            return Ok(FileLocation::from_path(v)?.into());
        }
        bail!("cannot parse {}", s)
    }
}
