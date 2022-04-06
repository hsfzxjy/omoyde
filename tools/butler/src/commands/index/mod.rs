use crate::prelude::*;
use clap::Args;

mod link;
use link::symlink_all_photos_to;

mod scanner;

#[derive(Args)]
pub(super) struct Index {}

impl Index {
    pub(super) fn run(self) -> Result<()> {
        let mut pt = pt_access_mut();
        pt.initialize(DEFAULT_PHOTOS_DB_PATH)?;
        scanner::Scanner::new(&mut pt, &mpt_access()).run()?;
        symlink_all_photos_to(&pt, ".butler/links/")?;
        pt.summary();
        pt.finalize(DEFAULT_PHOTOS_DB_PATH)?;
        Ok(())
    }
}
