use assert_cmd::Command;
use std::env;
use std::path::PathBuf;

#[test]
fn test_roundtrip_generated_code_rust() {
    let out_dir = env!("OUT_DIR");
    let output_rust = &PathBuf::from(&out_dir)
        .join("ion-code-gen")
        .join("Cargo.toml");
    let mut cmd = Command::new("cargo");
    cmd.args([
        "test",
        "--manifest-path",
        &output_rust.display().to_string(),
    ]);
    let command_assert = cmd.assert();
    command_assert.success();
}
