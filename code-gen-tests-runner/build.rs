// build.rs

use std::path::{Path, PathBuf};
use std::{env, fs};

use code_gen_core::generator::CodeGenerator;
use code_gen_core::utils::{JavaLanguage, RustLanguage};

fn main() -> std::io::Result<()> {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();

    let binding = PathBuf::from(&out_dir).join("java");
    let source_path = &binding.to_str().unwrap();

    let mut output_rust = PathBuf::from(&out_dir);
    generate_test_code_for("rust", &mut output_rust);
    println!(
        "cargo:warning=generated rust code for tests: {}",
        output_rust.to_str().unwrap()
    );

    // set up java project

    let mut output_java = PathBuf::from(&out_dir).join("java");
    generate_test_code_for("java", &mut output_java);
    println!(
        "cargo:warning=generated java code for tests {}",
        output_java.to_str().unwrap()
    );

    // Rerun java build if any source file changes, but then we'll check each file individually below
    println!("cargo:rerun-if-changed={}", source_path);
    println!("cargo:rustc-env=CLASSPATH=target/java");

    //TODO: This manual test generation can be removed once we have read-write APIs being generated for the data models

    let mut dest_path = Path::new(source_path)
        .join("ion-code-gen")
        .join("lib")
        .join("src")
        .join("test")
        .join("java")
        .join("ion_data_model");

    fs::create_dir_all(dest_path.as_path())?;

    dest_path.push("GeneratedCodeTests.java");
    fs::write(
        &dest_path,
        r#"
        package ion_data_model;

        public class GeneratedCodeTests {
            public static void main(String[] args) {
                StructWithFields s = new StructWithFields( 5, "hello", true);
                assert s.getA() == 5;
                assert s.getC();
                assert s.getB() == "hello";
            }
        }
        "#,
    )
    .unwrap();

    println!("cargo:warning=generated unit test for generated java code");

    Ok(())
}

fn generate_test_code_for(language: &str, output: &mut PathBuf) {
    let test_input_str = format!(
        "{}/code-gen-tests-runner/tests/input/",
        workspace_dir().as_os_str().to_str().unwrap()
    );

    let authority = PathBuf::from(format!(
        "{}/code-gen-tests-runner/tests/schema/",
        workspace_dir().as_os_str().to_str().unwrap()
    ));

    let authorities: Vec<&PathBuf> = vec![&authority];

    if language == "rust" {
        CodeGenerator::<RustLanguage>::new(output, authorities)
            .unwrap()
            .generate_code_with_tests_for_authorities(&test_input_str)
            .expect("Error generating code in Rust!");
    } else if language == "java" {
        //TODO: make a template for java tests
        CodeGenerator::<JavaLanguage>::new(output, authorities)
            .unwrap()
            .generate_code_with_tests_for_authorities(&test_input_str)
            .expect("Error generating code in Java!");
    } else {
        panic!("Programming language '{}' is not yet supported. Currently supported targets: 'java', 'rust'", language)
    }
}

fn workspace_dir() -> PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}
