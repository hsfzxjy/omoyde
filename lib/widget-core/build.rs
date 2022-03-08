use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

fn main() -> Result<(), std::io::Error> {
    let crate_dir = env!("CARGO_MANIFEST_DIR");
    let env_path = PathBuf::from(crate_dir).join(".env");

    println!("cargo:rerun-if-changed={}", env_path.display());

    let reader = BufReader::new(File::open(env_path)?);
    for res in reader.lines() {
        let line = res?;
        println!("cargo:rustc-env={}", line);

        if line.starts_with("OMOYDE_SYSTEM_WIDGET_ENCODING_RUST=") {
            let (_, encoding) = line.split_once('=').unwrap();
            match encoding {
                "utf8" => (),
                "utf16be" => (),
                _ => panic!("unsupported encoding: {}", encoding),
            };
            println!("cargo:rustc-cfg=storage_encoding=\"{}\"", encoding);
        }
    }
    Ok(())
}
