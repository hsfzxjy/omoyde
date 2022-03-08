use crate::prelude::*;
use clap::{AppSettings, Parser, Subcommand};

use crate::db;
use crate::db::Canonicalize;
use crate::generator;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(global_setting(AppSettings::PropagateVersion))]
#[clap(global_setting(AppSettings::UseLongFormatForHelpSubcommand))]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Mount {
        path: Option<PathBuf>,
        #[clap(short, long)]
        alias: Option<String>,
    },
    Umount {
        mpid: Uuid,
    },
    List {
        mpid: Option<Uuid>,
    },
    Index {},
    Commit {
        queries: Vec<db::PhotoQuery>,
        #[clap(short, long)]
        select: bool,
        #[clap(short, long)]
        unselect: bool,
        #[clap(short, long)]
        quiet: bool,
    },
    Fix {
        query: db::PhotoQuery,
        pref2: db::PhotoQuery,
    },
    Generate {
        #[clap(default_value_t = String::from("./assets/_generated"))]
        dest: String,
        #[clap(default_value_t = 85)]
        quality: u8,
        #[clap(short, long)]
        force: bool,
    },
}

pub fn handle_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Mount { path, alias } => {
            let mut mpt = db::MOUNTPOINT_TABLE.lock().unwrap();
            if let Some(p) = path {
                mpt.add_or_modify(p.resolve()?, alias);
            }
            print!("{}", mpt);
        }
        Commands::Umount { mpid } => {
            let mut mpt = db::MOUNTPOINT_TABLE.lock().unwrap();
            mpt.remove(mpid);
            print!("{}", mpt);
        }
        Commands::Index {} => {
            let mut pt = db::PHOTO_TABLE.lock().unwrap();
            pt.initialize_and_scan()?;
            pt.symlink_all_to_dir(".butler/links/")?;
            pt.summary();
            pt.finalize()?;
        }
        Commands::List { mpid } => {
            let mut pt = db::PHOTO_TABLE.lock().unwrap();
            pt.initialize()?;
            pt.display_list(mpid)?;
        }
        Commands::Commit {
            quiet,
            queries,
            select,
            unselect,
        } => {
            let mut pt = db::PHOTO_TABLE.lock().unwrap();
            pt.initialize()?;
            pt.commit(queries, db::CommitAction::from_cmd_args(select, unselect));
            if !quiet {
                pt.display_list(None)?;
            }
            pt.finalize()?;
        }
        Commands::Generate {
            quality,
            dest,
            force,
        } => {
            use rayon::prelude::*;
            let mut pt = db::PHOTO_TABLE.lock().unwrap();
            pt.initialize()?;
            let dest = PathBuf::from(dest);
            let metas = pt
                .pid2entry
                .values_mut()
                .filter_map(|entry| {
                    if entry.selected {
                        Some(generator::PhotoGenerator::new(entry, &dest, force, quality))
                    } else {
                        None
                    }
                })
                .collect::<Result<Vec<_>>>()?
                .into_par_iter()
                .map(|gen| gen.generate())
                .collect::<Result<Vec<_>>>()?;
            generator::write_bins(metas, dest.join("images.bin"))?;
        }
        Commands::Fix { query, pref2 } => {
            let mut pt = db::PHOTO_TABLE.lock().unwrap();
            pt.initialize()?;
            let metadata2 = pt.query_get_ref(pref2).unwrap().metadata.clone();
            let entry1 = pt.query_get_mut(query).unwrap();
            entry1.fix_metadata_from(metadata2);
            pt.finalize()?;
        }
    }

    Ok(())
}
