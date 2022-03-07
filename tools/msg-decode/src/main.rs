use anyhow::Result;
use msg_internal::{display_items, parse_items};
use std::env::args;
use std::fs::File;
use std::io::{BufReader, Read};

fn main() -> Result<()> {
    let path = args().nth(1).unwrap();
    let file = File::open(path)?;
    let mut buf = vec![];
    BufReader::new(file).read_to_end(&mut buf)?;
    display_items(&parse_items(&buf)?);
    Ok(())
}
