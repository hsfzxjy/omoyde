use super::entries::{DBPhotoEntry, LocalPhotoEntry};
use super::misc::*;
use super::table::PhotoTable;
use crate::prelude::*;

pub trait Stub {
    fn has_changed(&self) -> bool;
}

pub struct DBStub {
    pub pid: PID,
    pub file_hash: FileHash,
    pub status: PhotoEntryStatus,
    pub local_has_changed: bool,
}

impl Stub for DBStub {
    fn has_changed(&self) -> bool {
        self.local_has_changed
    }
}

pub trait SyncerQuery {
    type Stub: Stub;

    fn run_stage2(&mut self, stub: Option<Self::Stub>) -> Result<Option<Self::Stub>>;
}

impl SyncerQuery for LocalPhotoEntry {
    type Stub = DBStub;
    fn run_stage2(&mut self, mut stub: Option<DBStub>) -> Result<Option<DBStub>> {
        let should_hash = stub
            .as_ref()
            .map(|stub| stub.local_has_changed)
            .unwrap_or(true);
        if should_hash {
            self.fill_file_hash()?;
            stub.as_mut().map(|stub| {
                if stub.file_hash == self.file_hash.unwrap() {
                    stub.local_has_changed = false
                }
            });
        }
        Ok(stub)
    }
}

pub trait SyncerTarget {
    type Query: SyncerQuery;
    type Output;

    fn run_stage1<Q>(&self, query: Q) -> <Self::Query as SyncerQuery>::Stub
    where
        Q: Borrow<Self::Query>;
    fn run_finish<S>(&mut self, syncer: &S) -> Self::Output
    where
        S: Syncer<Query = Self::Query>;
}

impl SyncerTarget for DBPhotoEntry {
    type Query = LocalPhotoEntry;
    type Output = (PID, PhotoEntryStatus);

    fn run_stage1<Q>(&self, other: Q) -> DBStub
    where
        Q: Borrow<Self::Query>,
    {
        let other = other.borrow();
        let mdata = &self.metadata;
        let omdata = &other.metadata;
        let changed = mdata.ctime != omdata.ctime
            || mdata.mtime != omdata.mtime
            || mdata.file_length != omdata.file_length;
        DBStub {
            pid: self.pid,
            status: self.status,
            file_hash: self.file_hash,
            local_has_changed: changed,
        }
    }
    fn run_finish<S>(&mut self, syncer: &S) -> Self::Output
    where
        S: Syncer<Query = Self::Query>,
    {
        let changed = syncer.target_has_changed();
        if changed {
            let local_entry = syncer.get_query();
            self.metadata = local_entry.metadata.clone();
            self.file_hash = local_entry.file_hash.unwrap();
        }

        use PhotoEntryStatus::*;
        self.status = match (self.status, changed) {
            (Uncommitted, _) => Uncommitted,
            (Committed, true) => CommittedButMissing,
            (CommittedButMissing | CommittedButModified, true) => CommittedButModified,
            (_, false) => Committed,
        };

        (self.pid, self.status)
    }
}

pub trait SyncerTargetCollection {
    type Query: SyncerQuery;
    type Item: SyncerTarget<Query = Self::Query>;
    type SyncOutput;

    fn query_item<Q>(&self, query: Q) -> Option<&Self::Item>
    where
        Q: Borrow<Self::Query>;
    fn query_item_mut<Q>(&mut self, query: Q) -> Option<&mut Self::Item>
    where
        Q: Borrow<Self::Query>;
    fn sync_item<S>(
        &mut self,
        ret: <Self::Item as SyncerTarget>::Output,
        syncer: S,
    ) -> Result<Self::SyncOutput>
    where
        S: Syncer<Query = Self::Query>;
    fn add_item<S>(&mut self, syncer: S) -> Result<Self::SyncOutput>
    where
        S: Syncer<Query = Self::Query>;
}

impl SyncerTargetCollection for PhotoTable {
    type Query = LocalPhotoEntry;
    type Item = DBPhotoEntry;
    type SyncOutput = PID;

    fn query_item<Q>(&self, query: Q) -> Option<&Self::Item>
    where
        Q: Borrow<Self::Query>,
    {
        self.query_get_ref(query)
    }
    fn query_item_mut<Q>(&mut self, query: Q) -> Option<&mut Self::Item>
    where
        Q: Borrow<Self::Query>,
    {
        self.query_get_mut(query)
    }
    fn sync_item<S>(
        &mut self,
        ret: <Self::Item as SyncerTarget>::Output,
        syncer: S,
    ) -> Result<Self::SyncOutput>
    where
        S: Syncer<Query = Self::Query>,
    {
        self.index
            .mutate_status(ret.0, syncer.get_stub().unwrap().status, ret.1);
        Ok(ret.0)
    }
    fn add_item<S>(&mut self, syncer: S) -> Result<Self::SyncOutput>
    where
        S: Syncer<Query = Self::Query>,
    {
        self.insert_entry(syncer.take_query())
    }
}

pub trait Syncer: Sized {
    type Query: SyncerQuery;
    type Target: SyncerTarget<Query = Self::Query>;

    fn target_has_changed(&self) -> bool {
        self.get_stub()
            .map(|stub| stub.has_changed())
            .unwrap_or(true)
    }
    fn next_stage(&mut self);
    fn get_query(&self) -> &Self::Query;
    fn take_query(self) -> Self::Query;
    fn get_query_mut(&mut self) -> &mut Self::Query;
    fn get_stub(&self) -> Option<&<Self::Query as SyncerQuery>::Stub>;
    fn take_stub(&mut self) -> Option<<Self::Query as SyncerQuery>::Stub>;
    fn replace_stub(&mut self, stub: <Self::Query as SyncerQuery>::Stub);
    fn run_stage1<C>(mut self, db: &C) -> Self
    where
        C: SyncerTargetCollection<Query = Self::Query, Item = Self::Target>,
    {
        let query = self.get_query();
        let target = db.query_item(query);
        target
            .map(|t| t.run_stage1(query))
            .map(|stub| self.replace_stub(stub));
        self
    }
    fn run_stage2(mut self) -> Result<Self> {
        let stub = self.take_stub();
        let new_stub = self.get_query_mut().run_stage2(stub)?;
        new_stub.map(|stub| self.replace_stub(stub));
        self.next_stage();
        Ok(self)
    }
    fn run_finish<C>(self, db: &mut C) -> Result<C::SyncOutput>
    where
        C: SyncerTargetCollection<Query = Self::Query, Item = Self::Target>,
    {
        if self.get_stub().is_some() {
            let item = db.query_item_mut(self.get_query()).unwrap();
            let output = item.run_finish(&self);
            db.sync_item(output, self)
        } else {
            db.add_item(self)
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Stage {
    NotStarted,
    Stage1,
    Stage2,
    Finished,
}
use Stage::*;

impl Stage {
    fn next(&mut self) {
        *self = match self {
            NotStarted => Stage1,
            Stage1 => Stage2,
            Stage2 => Finished,
            Finished => panic!("{:?} has finished", self),
        };
    }
}

pub struct LocalSyncer {
    pub local_entry: Arc<LocalPhotoEntry>,
    pub stage: Stage,
    pub stub: Option<DBStub>,
}

impl LocalSyncer {
    pub fn new(local_entry: Arc<LocalPhotoEntry>) -> Self {
        Self {
            local_entry,
            stage: NotStarted,
            stub: None,
        }
    }
}

impl Syncer for LocalSyncer {
    type Query = LocalPhotoEntry;
    type Target = DBPhotoEntry;

    fn next_stage(&mut self) {
        self.stage.next();
    }
    fn get_query(&self) -> &Self::Query {
        self.local_entry.as_ref()
    }
    fn take_query(self) -> Self::Query {
        Arc::try_unwrap(self.local_entry).unwrap()
    }
    fn get_query_mut(&mut self) -> &mut Self::Query {
        Arc::get_mut(&mut self.local_entry).unwrap()
    }
    fn get_stub(&self) -> Option<&DBStub> {
        self.stub.as_ref()
    }
    fn take_stub(&mut self) -> Option<DBStub> {
        self.stub.take()
    }
    fn replace_stub(&mut self, stub: DBStub) {
        self.stub.replace(stub);
    }
}
