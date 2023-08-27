use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let generated_ts = include_str!(concat!(env!("OUT_DIR"), "/generated.ts"));

    let ts = format!(
        "export const EDMAN_UNIQUE_NAME = \"{}\";\n\n{}",
        edman_ce_adapter::EDMAN_UNIQUE_NAME,
        generated_ts
    );

    let file = std::env::args().skip(1).next().unwrap();
    std::fs::File::create(file)?.write_all(ts.as_bytes())?;

    Ok(())
}
