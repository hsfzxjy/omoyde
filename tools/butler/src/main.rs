#![feature(once_cell)]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate lazy_static;
extern crate fasthash;
extern crate num_cpus;

mod cmd;
mod db;
mod generator;
mod prelude;
mod util;

use prelude::*;

fn goto_work_directory() -> Result<()> {
    let mut curdir = env::current_dir()?;
    loop {
        let try_config_path = curdir.join("config.json");
        if try_config_path.exists() {
            break;
        };
        if let Some(p) = curdir.parent() {
            curdir = PathBuf::from(p);
        } else {
            panic!("cannot find root directory of project omoyde (the location that config.json lies in)");
        }
    }
    env::set_current_dir(&curdir)?;
    Ok(())
}

fn main() -> Result<()> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(num_cpus::get() * 2) // butler's job is light-weight enough, we thus double the thread pool capacity
        .build_global()
        .unwrap();
    goto_work_directory()?;
    db::initialize_mountpoint_table()?;
    cmd::handle_cli()?;
    db::finalize_mountpoint_table()?;
    Ok(())
}
