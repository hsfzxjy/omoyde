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
        pt.display_list(self.mpid)?;
        Ok(())
    }
}
