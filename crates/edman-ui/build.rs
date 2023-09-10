use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = PathBuf::from("../../proto/");

    tonic_build::configure()
        .build_server(false)
        .protoc_arg("--proto_path")
        .protoc_arg(proto_dir.to_str().unwrap())
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(
            &[proto_dir.join("ui.proto"), proto_dir.join("config.proto")],
            &["proto"],
        )?;

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
