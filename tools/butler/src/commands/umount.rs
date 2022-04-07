use crate::{prelude::*, util::tabled::print_table};
use clap::Args;

#[derive(Args)]
pub(super) struct Umount {
    mpid: Uuid,
}

impl Umount {
    pub(super) fn run(self) -> Result<()> {
        let mut mpt = mpt_access_mut();
        mpt.entry(self.mpid).remove();
        print_table(mpt.records());
        Ok(())
    }
}
