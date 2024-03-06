// build.rs

use std::path::{Path, PathBuf};
use std::{env, fs};

use code_gen_core::generator::CodeGenerator;
use code_gen_core::utils::{JavaLanguage, RustLanguage};
use ion_schema::authority::{DocumentAuthority, FileSystemDocumentAuthority};
use ion_schema::system::SchemaSystem;

use std::process::Command;

use walkdir::WalkDir;

fn main() -> std::io::Result<()> {
    let source_path = &format!(
        "{}/code-gen-tests-runner/tests/java/ion_data_model",
        workspace_dir().as_os_str().to_str().unwrap()
    );

    let target_path = &format!(
        "{}/code-gen-tests-runner/target/",
        workspace_dir().as_os_str().to_str().unwrap()
    );
    let output_rust = &PathBuf::from(format!(
        "{}/code-gen-tests-runner/tests/",
        workspace_dir().as_os_str().to_str().unwrap()
    ))
    .join("ion_data_model");
    generate_test_code_for("rust", output_rust);
    println!("cargo:warning=generated rust code for tests");

    let output_java = &PathBuf::from(format!(
        "{}/code-gen-tests-runner/tests/java/",
        workspace_dir().as_os_str().to_str().unwrap()
    ))
    .join("ion_data_model");
    generate_test_code_for("java", output_java);
    println!("cargo:warning=generated java code for tests");

    // Rerun java build if any source file changes, but then we'll check each file individually below
    println!("cargo:rerun-if-changed={}", source_path);
    println!("cargo:rustc-env=CLASSPATH=target/java");

    let target_dir = Path::new(target_path);

    for entry_result in WalkDir::new(source_path) {
        let entry = entry_result?;

        if let Some(extension) = entry.path().extension() {
            if extension == "java" {
                // check if the class file doesn't exist or is older
                let source = entry.into_path();

                // The target class file is basically the same path as the Java source file, relative to the target
                // directory
                let target = target_dir
                    .join(source.file_name().unwrap())
                    .with_extension("class");

                let build_file = BuildFile { source, target };

                if !file_up_to_date(&build_file)? {
                    build_java(&build_file, source_path, target_path)?;
                }
            }
        }
    }
    println!(
        "cargo:warning=built java code for tests {}",
        env!("CARGO_PKG_README")
    );

    // TODO: generate test files for generated java code and run tests for java roundtrip

    let dest_path = Path::new(source_path).join("GeneratedCodeTests.java");

    fs::write(
        &dest_path,
        r#"
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

// A simple holder for state on a given file
#[derive(Debug)]
struct BuildFile {
    source: PathBuf,
    target: PathBuf,
}

/// Determines whether the target file exists and is up-to-date by checking the last modified timestamp
fn file_up_to_date(BuildFile { source, target }: &BuildFile) -> std::io::Result<bool> {
    Ok(target.exists() && source.metadata()?.modified()? <= target.metadata()?.modified()?)
}

/// Executes javac to build the specified file
fn build_java(
    input: &BuildFile,
    source_path: &String,
    target_path: &String,
) -> std::io::Result<()> {
    let target_class_dir = Path::new(target_path).join("ion_data_model");
    let output = Command::new("javac")
        .args([
            "-d", // Specify the target directory for class files. Javac will create all parents if needed
            &target_class_dir.display().to_string(),
            "-sourcepath", // Specify where to find other source files (e.g. dependencies)
            source_path,
            input.source.to_str().unwrap(), // assuming that we're not dealing with weird filenames
        ])
        .output()?;

    if !output.status.success() {
        let stderr: String =
            String::from_utf8(output.stderr).expect("Unable to parse javac output");

        println!(
            "cargo:warning=Failed to build {:?}: {}",
            input.source, stderr
        );
    }

    Ok(())
}

fn generate_test_code_for(language: &str, output: &PathBuf) {
    let authorities: Vec<String> = vec![format!(
        "{}/code-gen-tests-runner/tests/schema/",
        workspace_dir().as_os_str().to_str().unwrap()
    )];

    // Set up document authorities vector
    let mut document_authorities: Vec<Box<dyn DocumentAuthority>> = vec![];

    for authority in &authorities {
        document_authorities.push(Box::new(FileSystemDocumentAuthority::new(Path::new(
            authority,
        ))))
    }

    // Create a new schema system from given document authorities
    let mut schema_system = SchemaSystem::new(document_authorities);

    // clean the target output directory if it already exists, before generating new code
    if output.exists() {
        fs::remove_dir_all(output).unwrap();
    }
    fs::create_dir_all(output).unwrap();
    let paths =
        fs::read_dir(PathBuf::from(&authorities[0])).expect("Couldn't read authorities directory");
    for schema_id in paths {
        let schema = schema_system
            .load_isl_schema(
                schema_id
                    .expect("Couldn't read schema file!")
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap(),
            )
            .unwrap();

        // generate code based on schema and programming language
        if language == "rust" {
            CodeGenerator::<RustLanguage>::new(output)
                .generate(schema)
                .expect("Error generating code!");
        } else if language == "java" {
            CodeGenerator::<JavaLanguage>::new(output)
                .generate(schema)
                .expect("Error generating code!");
        } else {
            panic!("Programming language '{}' is not yet supported. Currently supported targets: 'java', 'rust'", language)
        }
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
