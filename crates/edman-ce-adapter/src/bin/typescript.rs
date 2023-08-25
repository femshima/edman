use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ts = include_bytes!(concat!(env!("OUT_DIR"), "/generated.ts"));

    let file = std::env::args().skip(1).next().unwrap();
    std::fs::File::create(file)?.write_all(ts)?;

    Ok(())
}
