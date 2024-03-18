use crate::commands::IonCliCommand;
use anyhow::{bail, Result};
use clap::builder::TypedValueParser;
use clap::{Arg, ArgAction, ArgMatches, Command};
use code_gen_core::generator::CodeGenerator;
use code_gen_core::utils::{JavaLanguage, RustLanguage};
use ion_schema::authority::{DocumentAuthority, FileSystemDocumentAuthority};
use ion_schema::system::SchemaSystem;
use std::fs;
use std::path::{Path, PathBuf};

pub struct GenerateCommand;

impl IonCliCommand for GenerateCommand {
    fn name(&self) -> &'static str {
        "generate"
    }

    fn about(&self) -> &'static str {
        "Generates code using given schema file."
    }

    fn configure_args(&self, command: Command) -> Command {
        command
            .arg(
                Arg::new("output")
                    .long("output")
                    .short('o')
                    .help("Output directory [default: current directory]"),
            )
            .arg(
                Arg::new("schema")
                    .long("schema")
                    .short('s')
                    .help("Schema file"),
            )
            .arg(
                Arg::new("language")
                    .long("language")
                    .short('l')
                    .required(true)
                    .value_parser(["java", "rust"])
                    .help("Programming language for the generated code"),
            )
            .arg(
                // Directory(s) that will be used as authority(s) for schema system
                Arg::new("directory")
                    .long("directory")
                    .short('d')
                    // If this appears more than once, collect all values
                    .action(ArgAction::Append)
                    .value_name("DIRECTORY")
                    .required(true)
                    .help("One or more directories that will be searched for the requested schema"),
            )
    }

    fn run(&self, _command_path: &mut Vec<String>, args: &ArgMatches) -> Result<()> {
        // Extract programming language for code generation
        let language: &str = args.get_one::<String>("language").unwrap().as_str();

        // Extract output path information where the generated code will be saved
        // Create a module `ion_data_model` for storing all the generated code in the output directory
        let mut output = match args.get_one::<String>("output") {
            Some(output_path) => PathBuf::from(output_path),
            None => PathBuf::from("."),
        };

        // Extract the user provided document authorities/ directories
        let authorities_string: Vec<&String> = args.get_many("directory").unwrap().collect();
        let authorities: Vec<PathBuf> = authorities_string
            .iter()
            .map(|v| PathBuf::from(v))
            .collect();

        println!("Started generating code...");

        // Extract schema file provided by user
        match args.get_one::<String>("schema") {
            None => {
                // generate code based on schema and programming language
                match language {
                        "java" => CodeGenerator::<JavaLanguage>::new(&mut output, authorities.iter().collect())?.generate_code_for_authorities()?,
                        "rust" => CodeGenerator::<RustLanguage>::new(&mut output, authorities.iter().collect())?.generate_code_for_authorities()?,
                        _ => bail!(
                                "Programming language '{}' is not yet supported. Currently supported targets: 'java', 'rust'",
                                language
                            )
                    }
            }
            Some(schema_id) => {
                // generate code based on schema and programming language
                match language {
                    "java" => CodeGenerator::<JavaLanguage>::new(&mut output, authorities.iter().collect())?.generate_code_for(schema_id.to_string())?,
                    "rust" => CodeGenerator::<RustLanguage>::new(&mut output, authorities.iter().collect())?.generate_code_for(schema_id.to_string())?,
                    _ => bail!(
                            "Programming language '{}' is not yet supported. Currently supported targets: 'java', 'rust'",
                            language
                        )
                }
            }
        };

        println!("Code generation complete successfully!");
        println!("Path to generated code: {}", output.display());
        Ok(())
    }
}
