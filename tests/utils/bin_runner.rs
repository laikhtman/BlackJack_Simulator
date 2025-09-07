use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

pub fn run_freebj(args: &[&str]) -> Value {
    let mut bin = PathBuf::from(
        std::env::var("CARGO_TARGET_DIR").unwrap_or("target".to_owned()),
    );
    bin.push("debug/freebj");

    let proc = Command::new(bin)
        .args(args)
        .output()
        .expect("Couldn't launch freebj");

    if !proc.status.success() {
        panic!("freebj exited with status {}", proc.status);
    }

    serde_json::from_slice(&proc.stdout).expect("Couldn't parse JSON")
}
