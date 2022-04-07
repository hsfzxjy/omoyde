use crate::prelude::*;
use clap::{ArgGroup, Args};

#[derive(Args)]
#[clap(group(
    ArgGroup::new("selection")
        .arg("select")
        .conflicts_with("unselect")
))]
pub(super) struct Commit {
    queries: Vec<PhotoQuery>,
    #[clap(short, long)]
    select: bool,
    #[clap(short, long)]
    unselect: bool,
    #[clap(short, long)]
    quiet: bool,
}

impl Commit {
    pub(super) fn run(self) -> Result<()> {
        let mut pt = pt_access_mut();
        pt.initialize(DEFAULT_PHOTOS_DB_PATH)?;

        let select_request = match (self.select, self.unselect) {
            (true, true) => unreachable!(),
            (true, false) => Some(true),
            (false, true) => Some(false),
            _ => None,
        };

        for query in self.queries.into_iter() {
            let mut rec = unwrap_some_or!(pt.entry(query).modify().ok(), { continue });

            select_request.map(|sel| rec.set_selected(sel));
            rec.with_status(Committed).commit();
        }

        if !self.quiet {
            pt.display_list(None)?;
        }
        pt.finalize(DEFAULT_PHOTOS_DB_PATH)?;
        Ok(())
    }
}
