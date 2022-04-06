mod commit;
mod fix;
mod generate;
mod index;
mod list;
mod mount;
mod umount;

use clap::{AppSettings, Parser, Subcommand};
use paste::paste;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(global_setting(AppSettings::PropagateVersion))]
#[clap(global_setting(AppSettings::UseLongFormatForHelpSubcommand))]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

macro_rules! make {
    ( $( $name: ident),* ) => {
        $(
            paste!{
                use [< $name:lower >]::$name;
            }
        )*

        #[derive(Subcommand)]
        enum Commands {
            $( $name($name), )*
        }

        pub fn handle_cli() -> anyhow::Result<()> {
            let cli = Cli::parse();
            match cli.command {
                $( Commands::$name(x) => x.run()?, )*
            };
            Ok(())
        }
    };
}

make!(Mount, Umount, List, Index, Fix, Commit, Generate);
