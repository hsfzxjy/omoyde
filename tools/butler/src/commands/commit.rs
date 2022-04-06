use crate::prelude::*;
use clap::Args;

#[derive(Args)]
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

        for query in self.queries.into_iter() {
            let rec = unwrap_some_or!(pt.entry(query).modify().ok(), { continue });

            rec.set_selected_with(|x| {
                *x = match (self.select, self.unselect) {
                    (true, true) => panic!("--select cannot be used with --unselect"),
                    (true, false) => true,
                    (false, true) => false,
                    _ => return,
                };
            })
            .with_status(Committed)
            .commit();
        }

        if !self.quiet {
            pt.display_list(None)?;
        }
        pt.finalize(DEFAULT_PHOTOS_DB_PATH)?;
        Ok(())
    }
}
