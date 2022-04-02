use super::super::locations::*;
use super::entries::*;
use super::misc::*;
use super::synchronizer::*;
use crate::prelude::*;
use crate::util::serde::*;

#[derive(Debug, Default)]
pub struct PhotoTableIndex {
    loc2pid: HashMap<Arc<FileLocation>, u32>,
    selected_pids: HashSet<u32>,
    status2pids: HashMap<PhotoEntryStatus, HashSet<u32>>,
    mpid2selected_pids: HashMap<Uuid, HashSet<u32>>,
}

impl PhotoTableIndex {
    fn build_for(&mut self, entry: &DBPhotoEntry) {
        let pid = entry.pid;
        self.loc2pid.insert(entry.location.clone(), pid);
        if entry.selected {
            self.selected_pids.insert(entry.pid);
            self.mpid2selected_pids
                .entry(entry.location.mpid)
                .or_default()
                .insert(pid);
        }
        self.status2pids
            .entry(entry.status)
            .or_default()
            .insert(pid);
    }
    fn drop_for(&mut self, entry: &DBPhotoEntry) {
        let pid = entry.pid;
        self.loc2pid.remove(&entry.location);
        if entry.selected {
            self.selected_pids.remove(&pid);
            self.mpid2selected_pids
                .entry(entry.location.mpid)
                .or_default()
                .remove(&pid);
        }
        self.status2pids
            .entry(entry.status)
            .or_default()
            .remove(&pid);
    }
    pub fn mutate_status(
        &mut self,
        pid: PID,
        old_status: PhotoEntryStatus,
        new_status: PhotoEntryStatus,
    ) {
        if new_status != old_status {
            self.status2pids.entry(old_status).or_default().remove(&pid);
            self.status2pids.entry(new_status).or_default().insert(pid);
        }
    }
    fn flip_selected(&mut self, cur_entry: &DBPhotoEntry) {
        if cur_entry.selected {
            self.selected_pids.insert(cur_entry.pid);
            self.mpid2selected_pids
                .entry(cur_entry.location.mpid)
                .or_default()
                .insert(cur_entry.pid);
        } else {
            self.selected_pids.remove(&cur_entry.pid);
            self.mpid2selected_pids
                .entry(cur_entry.location.mpid)
                .or_default()
                .remove(&cur_entry.pid);
        }
    }
}
impl PhotoTableIndex {
    fn count_status(&mut self, status: PhotoEntryStatus) -> usize {
        self.status2pids.entry(status).or_default().len()
    }
    fn count_selected(&mut self, status: PhotoEntryStatus) -> usize {
        (&self.selected_pids & self.status2pids.entry(status).or_default()).len()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PhotoTable {
    counter: u32,
    pub pid2entry: BTreeMap<u32, DBPhotoEntry>,
    #[serde(skip)]
    pub index: PhotoTableIndex,
}

impl PhotoTable {
    pub fn summary(&mut self) {
        use PhotoEntryStatus::*;
        let ntotal = self.pid2entry.len();
        let nunc = self.index.count_status(Uncommitted);
        let ncsel = self.index.count_selected(Committed);
        let ndsel = self.index.count_selected(CommittedButMissing);
        let nmsel = self.index.count_selected(CommittedButModified);
        println!(
            "Among {ntotal} photos, {nc} reviewed.",
            ntotal = ntotal,
            nc = ntotal - nunc
        );
        println!(
            "Among {nc} reviewed photos, \
            {ncsel} selected, \
            {ndsel} selected but missing, \
            {nmsel} selected but modified.",
            nc = ntotal - nunc,
            ncsel = ncsel,
            ndsel = ndsel,
            nmsel = nmsel
        );
    }
    pub fn display_list(&mut self, mpid: Option<Uuid>) -> Result<()> {
        for entry in self.pid2entry.values_mut() {
            if let Some(uuid) = mpid {
                if entry.location.mpid != uuid {
                    continue;
                }
            }
            let mpid = entry.location.mpid;
            let fname = &entry.location.as_ref().filename;
            let exif_time = entry
                .metadata
                .etime
                .map(|t| t.timestamp().to_string())
                .unwrap_or_else(|| "none".to_string());
            let ctime = entry.metadata.ctime.timestamp().to_string();
            println!(
                "{} {} {} {:?} {} {} {} {}",
                entry.pid,
                mpid,
                fname.display(),
                entry.status,
                entry.selected,
                exif_time,
                ctime,
                entry
                    .commit_time
                    .map(|time| time.timestamp().to_string())
                    .unwrap_or("none".to_string())
            );
        }
        Ok(())
    }
}

impl PhotoTable {
    pub fn new() -> Self {
        Self {
            counter: 0,
            pid2entry: BTreeMap::new(),
            index: PhotoTableIndex::default(),
        }
    }
}

impl PhotoTable {
    pub fn build_index(&mut self) {
        for entry in self.pid2entry.values() {
            self.index.build_for(entry)
        }
    }

    pub fn insert_entry(&mut self, local_entry: LocalPhotoEntry) -> Result<PID> {
        let pid = &mut self.counter;
        *pid += 1;
        let entry = DBPhotoEntry::new(*pid, local_entry);
        self.index.build_for(&entry);
        self.pid2entry.insert(*pid, entry);
        Ok(*pid)
    }

    fn merge_from_local<I, I2>(&mut self, local_entries: I) -> Result<()>
    where
        I: IntoIterator<Item = I2> + Send,
        I2: IntoIterator<Item = Result<Arc<LocalPhotoEntry>>> + Send,
    {
        use std::ops::BitOr;
        use std::sync::mpsc::sync_channel;
        use std::thread;

        let existing_pids = local_entries
            .into_iter()
            .map(|entries| -> Result<_> {
                let (sx, rx) = sync_channel::<LocalSyncer>(2);

                let handle = thread::spawn(move || {
                    rx.into_iter()
                        .map(|syncer| syncer.run_stage2())
                        .collect::<Result<Vec<_>>>()
                });

                entries
                    .into_iter()
                    .map(|entry| -> Result<()> {
                        let mut syncer = LocalSyncer::new(entry?).run_stage1(self);
                        syncer.prefetch_stage2()?;
                        sx.send(syncer)?;
                        Ok(())
                    })
                    .collect::<Result<_>>()?;

                drop(sx);

                handle
                    .join()
                    .map_err(|_| anyhow!("fail to join thread"))??
                    .into_iter()
                    .map(|syncer| syncer.run_finish(self))
                    .collect::<Result<HashSet<_>>>()
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .fold(HashSet::<u32>::new(), |x, y| x.bitor(&y));

        self.pid2entry.retain(|pid, entry| {
            if existing_pids.contains(pid) {
                return true;
            }
            let should_remove = entry.handle_local_missing();
            if should_remove {
                self.index.drop_for(entry);
            }
            should_remove
        });
        Ok(())
    }
}

impl PhotoTable {
    pub fn query_pid<Q: Into<PhotoQuery>>(&self, query: Q) -> Option<PID> {
        let query = query.into();
        match query {
            PhotoQuery::PID(pid) => Some(pid.clone()),
            PhotoQuery::FileLocation(ref loc) => self.index.loc2pid.get(loc).cloned(),
        }
    }
    pub fn query_get_ref<Q: Into<PhotoQuery>>(&self, query: Q) -> Option<&DBPhotoEntry> {
        self.query_pid(query)
            .and_then(|pid| self.pid2entry.get(&pid))
    }
    pub fn query_get_mut<Q: Into<PhotoQuery>>(&mut self, query: Q) -> Option<&mut DBPhotoEntry> {
        self.query_pid(query)
            .and_then(|pid| self.pid2entry.get_mut(&pid))
    }
}

impl PhotoTable {
    pub fn initialize(&mut self) -> Result<()> {
        self.load_from_path(".butler/photos")?;
        self.build_index();
        Ok(())
    }
    pub fn initialize_and_scan(&mut self) -> Result<()> {
        self.initialize()?;
        let local_entries = super::ops::scan_photos()?;
        self.merge_from_local(local_entries)?;
        Ok(())
    }
    pub fn finalize(&self) -> Result<()> {
        self.save_to_path(".butler/photos")?;
        Ok(())
    }
    pub fn symlink_all_to_dir<P: AsRef<Path>>(&mut self, dir: P) -> Result<()> {
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir)?;
        for entry in self.pid2entry.values_mut() {
            let dst = PathBuf::new().join(&dir).join(format!(
                "{}.{}",
                entry.pid,
                entry
                    .location
                    .filename
                    .extension()
                    .unwrap()
                    .to_string_lossy()
            ));
            std::os::unix::fs::symlink(entry.location.filepath(), dst.as_path())?;
        }
        Ok(())
    }

    pub fn commit(&mut self, queries: impl IntoIterator<Item = PhotoQuery>, action: CommitAction) {
        use CommitAction::*;
        use PhotoEntryStatus::*;
        for query in queries.into_iter() {
            let pid = self.query_pid(query);
            let entry = if let Some(pid) = pid {
                self.pid2entry.get_mut(&pid).unwrap()
            } else {
                continue;
            };
            match action {
                Select => {
                    entry.selected = true;
                    self.index.flip_selected(entry);
                }
                Unselect => {
                    entry.selected = false;
                    self.index.flip_selected(entry);
                }
                _ => {}
            };
            let old_status = entry.status;
            entry.status = Committed;
            if old_status != Committed {
                self.index
                    .mutate_status(entry.pid, old_status, entry.status);
                entry.commit_time = Some(Utc::now());
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum CommitAction {
    CommitOnly,
    Select,
    Unselect,
}

impl CommitAction {
    pub fn from_cmd_args(select: bool, unselect: bool) -> Self {
        use CommitAction::*;
        match (select, unselect) {
            (true, false) => Select,
            (false, true) => Unselect,
            (false, false) => CommitOnly,
            _ => panic!("Ambigious"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum PhotoQuery {
    PID(PID),
    FileLocation(Arc<FileLocation>),
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
    T: Borrow<LocalPhotoEntry>,
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

lazy_static! {
    pub static ref PHOTO_TABLE: Mutex<PhotoTable> = Mutex::new(PhotoTable::new());
}
