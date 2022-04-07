use super::util::display::print_photos;
use crate::prelude::*;
use clap::Args;

#[derive(Args)]
pub(super) struct List {
    mpid: Option<Uuid>,
}

impl List {
    pub(super) fn run(self) -> Result<()> {
        let pt = pt_access_mut();
        pt.initialize(DEFAULT_PHOTOS_DB_PATH)?;

        let mpid = self.mpid.as_ref();
        print_photos(
            pt.records()
                .filter(|rec| mpid.map(|mpid| mpid == &rec.location.mpid).unwrap_or(true)),
        );
        Ok(())
    }
}
