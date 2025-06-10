use std::env;
use std::path::Path;

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let script = root
        .join("pi.ld")
        .canonicalize()
        .expect("Failed to find linker script");

    println!("cargo:rerun-if-changed={}", script.display());
    println!("cargo:rustc-link-arg=--script={}", script.display());
}
