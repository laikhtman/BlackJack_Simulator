use serde_json::Value;
use std::process::Command;

pub fn run_freebj(args: &[&str]) -> Value {
    let proc = Command::new(env!("CARGO_BIN_EXE_freebj"))
        .args(args)
        .output()
        .expect("Couldn't launch freebj");

    if !proc.status.success() {
        panic!("freebj exited with status {}: {}", proc.status,
            String::from_utf8_lossy(&proc.stderr));
    }

    serde_json::from_slice(&proc.stdout).expect("Couldn't parse JSON")
}
