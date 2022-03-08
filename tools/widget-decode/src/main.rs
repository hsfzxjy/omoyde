use anyhow::Result;
use std::env::args;
use std::fs::File;
use std::io::{BufReader, Read};
use widget_core::{display_widgets, parse_widgets};

fn main() -> Result<()> {
    let path = args().nth(1).unwrap();
    let file = File::open(path)?;
    let mut buf = vec![];
    BufReader::new(file).read_to_end(&mut buf)?;
    display_widgets(&parse_widgets(&buf)?);
    Ok(())
}
