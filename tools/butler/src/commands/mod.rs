mod commit;
mod fix;
mod generate;
mod index;
mod list;
mod mount;
mod umount;
mod util;

use anyhow::Result;
use clap::{CommandFactory, ErrorKind, Parser, Subcommand};
use paste::paste;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long, parse(from_occurrences), help = "Less verbosity")]
    quiet: usize,

    #[clap(short, long, parse(from_occurrences), help = "More verbosity")]
    verbose: usize,

    #[clap(subcommand)]
    commands: Commands,
}

pub fn handle_cli() -> Result<()> {
    let cli = Cli::parse();
    init_logger(&cli);
    init_yansi();
    dispatch_subcommands(cli.commands)
}

pub fn init_yansi() {
    use atty::*;
    use yansi::*;
    if !is(Stream::Stdout) {
        Paint::disable();
    }
}

fn init_logger(cli: &Cli) {
    use simplelog::*;

    let verbosity: i8 = {
        let default = 3i8;
        let verboser = cli.verbose as i8;
        let quieter = cli.quiet as i8;
        if verboser != 0 && quieter != 0 {
            Cli::command()
                .error(
                    ErrorKind::ArgumentConflict,
                    "cannot use --quiet with --verbose",
                )
                .exit()
        }
        default + verboser - quieter
    };

    CombinedLogger::init(vec![TermLogger::new(
        match verbosity {
            i8::MIN..=0 => LevelFilter::Off,
            1 => LevelFilter::Error,
            2 => LevelFilter::Warn,
            3 => LevelFilter::Info,
            4 => LevelFilter::Debug,
            5.. => LevelFilter::Trace,
        },
        {
            let mut builder = ConfigBuilder::new();
            builder.set_time_level(LevelFilter::Trace);
            builder.set_time_to_local(true);
            builder.build()
        },
        TerminalMode::Mixed,
        {
            use atty::*;
            if is(Stream::Stdout) && is(Stream::Stderr) {
                ColorChoice::Auto
            } else {
                ColorChoice::Never
            }
        },
    )])
    .unwrap()
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

        fn dispatch_subcommands(subcommands: Commands) -> Result<()> {
            match subcommands {
                $( Commands::$name(x) => x.run()?, )*
            };
            Ok(())
        }
    };
}

make!(Mount, Umount, List, Index, Fix, Commit, Generate);
