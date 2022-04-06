use crate::prelude::*;
use clap::Args;

#[derive(Args)]
pub(super) struct Fix {
    dst: PhotoQuery,
    src: PhotoQuery,
}

impl Fix {
    pub(super) fn run(self) -> Result<()> {
        let mut pt = pt_access_mut();
        pt.initialize(DEFAULT_PHOTOS_DB_PATH)?;
        let metadata2 = pt.query(self.dst).unwrap().metadata.clone();

        pt.entry(self.src)
            .modify()
            .unwrap()
            .set_metadata_with(|mdata| mdata.fix_from(&metadata2))
            .commit();
        pt.finalize(DEFAULT_PHOTOS_DB_PATH)?;
        Ok(())
    }
}
