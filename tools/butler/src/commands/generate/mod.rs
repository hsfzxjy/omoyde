use crate::prelude::*;
use clap::Args;

mod generator;
use generator::PhotoGenerator;

#[derive(Args)]
pub(super) struct Generate {
    #[clap(default_value_t = String::from("./assets/_generated"))]
    dest: String,
    #[clap(default_value_t = 85)]
    quality: u8,
    #[clap(short, long)]
    force: bool,
}

impl Generate {
    pub(super) fn run(self) -> Result<()> {
        use rayon::prelude::*;
        let pt = pt_access_mut();
        pt.initialize(DEFAULT_PHOTOS_DB_PATH)?;
        let dest = PathBuf::from(self.dest);
        let metas = pt
            .records()
            .filter(|entry| entry.selected)
            .map(|entry| generator::PhotoGenerator::new(entry, &dest, self.force, self.quality))
            .collect::<Result<Vec<_>>>()?
            .into_par_iter()
            .map(PhotoGenerator::generate)
            .collect::<Result<Vec<_>>>()?;
        generator::write_bins(metas, dest.join("images.bin"))?;
        Ok(())
    }
}
