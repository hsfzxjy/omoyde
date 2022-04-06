use crate::prelude::*;

#[derive(Debug, Default)]
pub struct PhotoTableIndex {
    pub(super) loc2pid: HashMap<Arc<FileLocation>, u32>,
    selected_pids: HashSet<u32>,
    status2pids: HashMap<PhotoRecordStatus, HashSet<u32>>,
    mpid2selected_pids: HashMap<Uuid, HashSet<u32>>,
}

impl PhotoTableIndex {
    fn curate_selected(&mut self, rec: &PhotoRecord, target: bool) {
        let pid = rec.pid;
        let mpid2sel_entry = self
            .mpid2selected_pids
            .entry(rec.location.mpid)
            .or_default();
        if target {
            self.selected_pids.insert(pid);
            mpid2sel_entry.insert(pid);
        } else {
            self.selected_pids.remove(&pid);
            mpid2sel_entry.remove(&pid);
        }
    }
}

impl PhotoTableIndex {
    pub(super) fn build_for(&mut self, rec: &PhotoRecord) {
        let pid = rec.pid;
        self.loc2pid.insert(rec.location.clone(), pid);
        self.curate_selected(rec, rec.selected);
        self.status2pids.entry(rec.status).or_default().insert(pid);
    }
    pub(super) fn drop_for(&mut self, rec: &PhotoRecord) {
        let pid = rec.pid;
        self.loc2pid.remove(&rec.location);
        self.curate_selected(rec, false);
        self.status2pids.entry(rec.status).or_default().remove(&pid);
    }
    pub(super) fn mutate_status(
        &mut self,
        pid: PID,
        old_status: PhotoRecordStatus,
        new_status: PhotoRecordStatus,
    ) {
        self.status2pids.entry(old_status).or_default().remove(&pid);
        self.status2pids.entry(new_status).or_default().insert(pid);
    }
    pub(super) fn flip_selected(&mut self, rec: &PhotoRecord) {
        self.curate_selected(rec, rec.selected);
    }
}
impl PhotoTableIndex {
    pub(super) fn count_status(&mut self, status: PhotoRecordStatus) -> usize {
        self.status2pids.entry(status).or_default().len()
    }
    pub(super) fn count_selected(&mut self, status: PhotoRecordStatus) -> usize {
        (&self.selected_pids & self.status2pids.entry(status).or_default()).len()
    }
}
