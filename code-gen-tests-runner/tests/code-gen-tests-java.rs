use assert_cmd::Command;
use std::path::Path;

#[test]
fn test_roundtrip_generated_code_java() {
    let dest_path = Path::new("./tests/java/ion_data_model").join("GeneratedCodeTests.java");

    let target_class_path = Path::new("./target").join("ion_data_model");
    let mut cmd = Command::new("java");
    cmd.args([
        "-ea",
        "-cp",
        &target_class_path.display().to_string(),
        &dest_path.display().to_string(),
    ]);
    let command_assert = cmd.assert();
    command_assert.success();
}
