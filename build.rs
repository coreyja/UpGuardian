use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=frontend/**/*");
    Command::new("bun")
        .args(["build", "frontend/index.ts", "--outdir", "public/frontend/"])
        .status()
        .expect("Failed to build frontend");
}
