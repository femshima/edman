use std::{env, path::Path, process::Command};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../prisma");

    let out_dir = env::var("OUT_DIR").unwrap();
    let codegen_dest_path = Path::new(&out_dir).join("generated.rs");

    env::set_var("PRISMA_OUT_FILE", codegen_dest_path);
    env::set_var("CARGO_TARGET_DIR", &out_dir);

    let mut cargo = Command::new("cargo")
        .args(&["run", "-p", "prisma-cli", "generate"])
        .current_dir("../../")
        .spawn()
        .unwrap();

    cargo.wait().expect("Failed run to codegen command");
}
