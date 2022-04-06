#![feature(once_cell)]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate lazy_static;
extern crate num_cpus;

mod _vendors;
mod commands;
mod consts;
mod db;
mod locations;
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
    goto_work_directory()?;
    mpt_access_mut().initialize(DEFAULT_MOUNTPOINTS_DB_PATH)?;
    commands::handle_cli()?;
    mpt_access().finalize(DEFAULT_MOUNTPOINTS_DB_PATH)
}
