fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure()
        .build_client(false)
        .protoc_arg("--proto_path")
        .protoc_arg(std::fs::canonicalize("../../proto/")?.to_str().unwrap())
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile(&["chrome_extension.proto"], &["proto"])?;

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}
