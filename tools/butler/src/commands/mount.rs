use crate::prelude::*;
use clap::Args;

#[derive(Args)]
pub(super) struct Mount {
    path: Option<PathBuf>,
    #[clap(short, long)]
    alias: Option<String>,
}

impl Mount {
    pub(super) fn run(self) -> Result<()> {
        let mut mpt = mpt_access_mut();
        if let Some(p) = self.path {
            mpt.insert_or_update(p.resolve(), self.alias);
        }
        print!("{}", mpt);
        Ok(())
    }
}
