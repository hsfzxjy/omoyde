#![feature(once_cell)]

#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate lazy_static;
extern crate fasthash;

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
    // println!("cd to: {}", curdir.display());
    Ok(())
}

fn main() -> Result<()> {
    goto_work_directory()?;
    db::initialize_mountpoint_table()?;
    cmd::handle_cli()?;
    db::finalize_mountpoint_table()?;
    Ok(())
}
