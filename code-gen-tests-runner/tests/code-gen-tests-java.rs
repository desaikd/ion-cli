use assert_cmd::Command;
use std::path::PathBuf;

#[test]
fn test_roundtrip_generated_code_java() {
    let out_dir = env!("OUT_DIR");
    let dest_path = &PathBuf::from(&out_dir).join("java").join("ion-code-gen");

    let mut cmd = Command::new("gradle");
    cmd.args(["test", "--project-dir", &dest_path.display().to_string()]);
    let command_assert = cmd.assert();
    command_assert.success();
}
