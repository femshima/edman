use std::path::{Path, PathBuf};

use typeshare_core::{language::Language, parser::ParsedData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);

    tonic_build::configure()
        .build_server(false)
        .message_attribute(".", "#[::typeshare::typeshare]")
        .enum_attribute(".", "#[::typeshare::typeshare]")
        .message_attribute(".", "#[derive(::serde::Serialize, ::serde::Deserialize)]")
        .enum_attribute(".", "#[derive(::serde::Serialize, ::serde::Deserialize)]")
        .protoc_arg("--proto_path")
        .protoc_arg(std::fs::canonicalize("../../proto/")?.to_str().unwrap())
        .protoc_arg("--experimental_allow_proto3_optional")
        .out_dir(&out_dir)
        .compile(&["chrome_extension.proto"], &["proto"])?;

    let mut typescript = typeshare_core::language::TypeScript::default();
    let parsed_data = parse_files(&[
        Path::new("src/main.rs"),
        &out_dir.join("chrome_extension.rs"),
    ]);
    let mut type_file = std::fs::File::create(out_dir.join("generated.ts"))?;
    typescript.generate_types(&mut type_file, &parsed_data)?;

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn parse_files<P: AsRef<Path>>(paths: &[P]) -> ParsedData {
    paths
        .iter()
        .map(|path| {
            let content = std::fs::read_to_string(path).unwrap();
            typeshare_core::parser::parse(&content).unwrap()
        })
        .fold(ParsedData::default(), |mut identity, other| {
            identity.add(other);
            identity
        })
}