//! Build script: generates Rust code from FlatBuffers schemas.
//!
//! Requires `flatc` on PATH (`brew install flatbuffers`).

use std::process::Command;

fn main() {
    let schemas = &["flatbufs/rpc.fbs", "flatbufs/transform_tree.fbs"];
    let out_dir = "src/gen";

    std::fs::create_dir_all(out_dir).expect("failed to create src/gen");

    let status = Command::new("flatc")
        .args(["--rust", "-o", out_dir])
        .args(schemas)
        .status()
        .expect("flatc not found — run: brew install flatbuffers");

    assert!(status.success(), "flatc failed");

    for schema in schemas {
        println!("cargo:rerun-if-changed={schema}");
    }
}
