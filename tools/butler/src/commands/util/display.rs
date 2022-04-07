use tabled::Tabled;

use crate::prelude::{yansi::*, *};
use crate::util::tabled::print_table;

#[derive(Tabled)]
struct PhotoRecordForDisplay<'a> {
    #[tabled(rename = "PID")]
    pid: &'a PID,
    #[tabled(rename = "PATH", display_with = "display_location")]
    location: &'a Arc<FileLocation>,
    #[tabled(rename = "Status", display_with = "display_status")]
    status: &'a PhotoRecordStatus,
    #[tabled(rename = "Selected", display_with = "display_bool")]
    selected: &'a bool,
}

impl<'a> PhotoRecordForDisplay<'a> {
    fn new(rec: &'a PhotoRecord) -> Self {
        Self {
            pid: &rec.pid,
            location: &rec.location,
            status: &rec.status,
            selected: &rec.selected,
        }
    }
}

fn display_bool(val: &bool) -> String {
    match val {
        true => Paint::green("Yes"),
        false => Paint::new("No").dimmed(),
    }
    .to_string()
}

fn display_location(loc: &Arc<FileLocation>) -> String {
    loc.filename.display().to_string()
}

fn display_status(status: &PhotoRecordStatus) -> String {
    match status {
        Committed => Paint::green("committed"),
        Uncommitted => Paint::new("untracked").dimmed(),
        CommittedButModified => Paint::yellow("modified"),
        CommittedButMissing => Paint::red("missing"),
    }
    .to_string()
}

pub fn print_photos<'a, I>(iter: I)
where
    I: IntoIterator<Item = &'a PhotoRecord>,
{
    print_table(iter.into_iter().map(PhotoRecordForDisplay::new))
}
