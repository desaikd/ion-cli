use crate::context::{AbstractDataType, CodeGenContext};
use crate::result::CodeGenError::IoError;
use crate::result::{invalid_abstract_data_type_error, CodeGenResult};
use crate::utils::{Field, Import, JavaLanguage, Language, RustLanguage};
use crate::utils::{IonSchemaType, Template};
use convert_case::{Case, Casing};
use ion_schema::authority::{DocumentAuthority, FileSystemDocumentAuthority};
use ion_schema::isl::isl_constraint::{IslConstraint, IslConstraintValue};
use ion_schema::isl::isl_type::IslType;
use ion_schema::isl::isl_type_reference::IslTypeRef;
use ion_schema::system::SchemaSystem;
use std::collections::HashMap;
use std::fs;
use std::fs::{DirEntry, File};
use std::io::{Error, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::process::Command;
use tera::{Context, Tera};

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

// TODO: Add generate_code_with_tests_for(schema) - JavaLanguage, RustLanguage
pub struct CodeGenerator<'a, L: Language + 'a> {
    // Represents the templating engine - tera
    // more information: https://docs.rs/tera/latest/tera/
    pub(crate) tera: Tera,
    output: &'a mut PathBuf,
    // Represents current test input Ion file that can be used to generate test
    current_test_input: Option<PathBuf>,
    // Represents the schema system that is used by the code generator to load schema
    schema_system: SchemaSystem,
    // Represents the document authorities stored in the schema system
    //TODO: remove this property once the below issue is resolved in `ion-schema-rust`: https://github.com/amazon-ion/ion-schema-rust/issues/212
    authorities: Vec<&'a PathBuf>,
    // Represents a counter for naming anonymous type definitions
    pub(crate) anonymous_type_counter: usize,
    // Current type definition is root type or not
    is_root_type: bool,
    phantom: PhantomData<L>,
}

impl<'a> CodeGenerator<'a, RustLanguage> {
    pub fn new(
        output: &'a mut PathBuf,
        authorities: Vec<&'a PathBuf>,
    ) -> CodeGenResult<CodeGenerator<'a, RustLanguage>> {
        // set up document authorities vector
        let mut document_authorities: Vec<Box<dyn DocumentAuthority>> = vec![];

        for authority in &authorities {
            document_authorities.push(Box::new(FileSystemDocumentAuthority::new(Path::new(
                authority,
            ))))
        }

        Ok(Self {
            output: {
                // create `ion_data_model` module inside the output directory where all the generated code will be stored.
                output.push("ion_data_model");

                // clean the target output directory if it already exists, before generating new code
                if output.as_path().exists() {
                    fs::remove_dir_all(output.as_path())?;
                }
                fs::create_dir_all(output.as_path())?;
                output
            },
            current_test_input: None,
            // creates a schema system with given authorities
            schema_system: SchemaSystem::new(document_authorities),
            authorities,
            anonymous_type_counter: 0,
            tera: {
                Tera::new(&format!(
                    "{}/code-gen-core/src/templates/rust/*.templ",
                    workspace_dir().as_os_str().to_str().unwrap()
                ))?
            },
            phantom: PhantomData,
            is_root_type: true,
        })
    }
}

impl<'a> CodeGenerator<'a, JavaLanguage> {
    pub fn new(
        output: &'a mut PathBuf,
        authorities: Vec<&'a PathBuf>,
    ) -> CodeGenResult<CodeGenerator<'a, JavaLanguage>> {
        // set up document authorities vector
        let mut document_authorities: Vec<Box<dyn DocumentAuthority>> = vec![];

        for authority in &authorities {
            document_authorities.push(Box::new(FileSystemDocumentAuthority::new(Path::new(
                authority,
            ))))
        }

        Ok(Self {
            output: {
                // create `ion_data_model` module inside the output directory where all the generated code will be stored.
                output.push("ion_data_model");

                // clean the target output directory if it already exists, before generating new code
                if output.as_path().exists() {
                    fs::remove_dir_all(output.as_path())?;
                }
                fs::create_dir_all(output.as_path())?;
                output
            },
            current_test_input: None,
            // creates a schema system with given authorities
            schema_system: SchemaSystem::new(document_authorities),
            authorities,
            anonymous_type_counter: 0,
            tera: Tera::new(&format!(
                "{}/code-gen-core/src/templates/java/*.templ",
                workspace_dir().as_os_str().to_str().unwrap()
            ))
            .unwrap(),
            phantom: PhantomData,
            is_root_type: true,
        })
    }
}

impl<'a, L: Language + 'static> CodeGenerator<'a, L> {
    /// Represents a [tera] filter that converts given tera string value to [upper camel case].
    /// Returns error if the given value is not a string.
    ///
    /// For more information: <https://docs.rs/tera/1.19.0/tera/struct.Tera.html#method.register_filter>
    ///
    /// [tera]: <https://docs.rs/tera/latest/tera/>
    /// [upper camel case]: <https://docs.rs/convert_case/latest/convert_case/enum.Case.html#variant.UpperCamel>
    pub fn upper_camel(
        value: &tera::Value,
        _map: &HashMap<String, tera::Value>,
    ) -> Result<tera::Value, tera::Error> {
        Ok(tera::Value::String(
            value
                .as_str()
                .ok_or(tera::Error::msg(
                    "the `upper_camel` filter only accepts strings",
                ))?
                .to_case(Case::UpperCamel),
        ))
    }

    /// Represents a [tera] filter that converts given tera string value to [camel case].
    /// Returns error if the given value is not a string.
    ///
    /// For more information: <https://docs.rs/tera/1.19.0/tera/struct.Tera.html#method.register_filter>
    ///
    /// [tera]: <https://docs.rs/tera/latest/tera/>
    /// [camel case]: <https://docs.rs/convert_case/latest/convert_case/enum.Case.html#variant.Camel>
    pub fn camel(
        value: &tera::Value,
        _map: &HashMap<String, tera::Value>,
    ) -> Result<tera::Value, tera::Error> {
        Ok(tera::Value::String(
            value
                .as_str()
                .ok_or(tera::Error::msg("Required string for this filter"))?
                .to_case(Case::Camel),
        ))
    }

    /// Represents a [tera] filter that converts given tera string value to [snake case].
    /// Returns error if the given value is not a string.
    ///
    /// For more information: <https://docs.rs/tera/1.19.0/tera/struct.Tera.html#method.register_filter>
    ///
    /// [tera]: <https://docs.rs/tera/latest/tera/>
    /// [snake case]: <https://docs.rs/convert_case/latest/convert_case/enum.Case.html#variant.Camel>
    pub fn snake(
        value: &tera::Value,
        _map: &HashMap<String, tera::Value>,
    ) -> Result<tera::Value, tera::Error> {
        Ok(tera::Value::String(
            value
                .as_str()
                .ok_or(tera::Error::msg("Required string for this filter"))?
                .to_case(Case::Snake),
        ))
    }

    /// Represents a [tera] filter that return true if the value is a built in type, otherwise returns false.
    ///
    /// For more information: <https://docs.rs/tera/1.19.0/tera/struct.Tera.html#method.register_filter>
    ///
    /// [tera]: <https://docs.rs/tera/latest/tera/>
    pub fn is_built_in_type(
        value: &tera::Value,
        _map: &HashMap<String, tera::Value>,
    ) -> Result<tera::Value, tera::Error> {
        Ok(tera::Value::Bool(L::is_built_in_type(
            value.as_str().ok_or(tera::Error::msg(
                "`is_built_in_type` called with non-String Value",
            ))?,
        )))
    }

    /// Returns true if its a built in type for ISL otherwise returns false
    pub fn is_built_in_isl_type(&self, name: &str) -> bool {
        matches!(
            name,
            "int" | "string" | "bool" | "float" | "symbol" | "blob" | "clob"
        )
    }

    /// Generates code in Rust for given Ion Schema id
    pub fn generate_code_for(&mut self, schema_id: String) -> CodeGenResult<()> {
        self.generate(schema_id)
    }

    /// Generates code with tests in Rust for given Ion Schema id
    pub fn generate_code_with_tests_for(
        &mut self,
        schema_id: String,
        test_input: PathBuf,
    ) -> CodeGenResult<()> {
        // set the test case input
        self.current_test_input = Some(test_input);
        self.generate(schema_id)
    }

    /// Generates code in Rust for schemas in given authorities
    pub fn generate_code_for_authorities(&mut self) -> CodeGenResult<()> {
        let paths = fs::read_dir(self.authorities[0])?;

        for schema_file in paths {
            self.is_root_type = true;

            let schema_id = Self::schema_id_from(schema_file?)?;
            // generate code based on schema and programming language
            self.generate(schema_id)?;
        }
        Ok(())
    }

    /// Generates code with tests in Rust for schemas. Use given test input folder
    /// which has test case Ion files with same filename as each schema in the authorities.
    /// Note: Generated code will have a single module for all the data models created for schemas in given authorities.
    // This method wraps the generated code into a crate or gradle project and is used only by the build script
    pub fn generate_code_with_tests_for_authorities(
        &mut self,
        test_input_str: &str,
    ) -> CodeGenResult<()> {
        // remove the previously added `ion_data_model`, then add `ion-code-gen/ion_data_model` since this method generates a crate to be used for testing
        fs::remove_dir(self.output.as_path())?;
        self.output.pop();
        self.output.push("ion-code-gen");
        if self.output.exists() {
            fs::remove_dir_all(self.output.as_path())?;
        }
        fs::create_dir_all(self.output.as_path())?;

        if L::requires_modules() {
            self.output.push("src");
            self.output.push("ion_data_model");
            fs::create_dir_all(self.output.as_path())?;
        } else {
            //TODO: initialize gradle project and generate ion_data_model inside src/main/java for generated classes
            // generate tests in test/main/java/ion_data_model

            let output = Command::new("gradle")
                .args([
                    "init",
                    "--use-defaults",
                    "--type",
                    "java-library",
                    "--project-dir",
                    &format!("{}", self.output.display()),
                ])
                .output()?;
            if output.status.success() {
                self.output.push("lib");
                self.output.push("src");
                self.output.push("main");
                self.output.push("java");
                self.output.push("ion_data_model");
                fs::create_dir_all(self.output.as_path())?;
            } else {
                std::io::stderr().write_all(&output.stderr).unwrap();
                return Err(IoError {
                    source: Error::other(
                        "Error generating gradle project for Java code generation with tests",
                    ),
                });
            }
        }

        let paths = fs::read_dir(self.authorities[0])?;

        for schema_file in paths {
            self.is_root_type = true;
            let schema_id = Self::schema_id_from(schema_file?)?;
            let input_file_name = &schema_id.replace("isl", "ion");
            let test_input = Path::new(test_input_str).join(input_file_name);

            // generate code based on schema with test based on current test input
            self.generate_code_with_tests_for(schema_id, test_input)?;
        }

        if L::requires_modules() {
            let rendered_cargo_toml = self.tera.render("Cargo.toml.templ", &Context::new())?;
            let mut cargo_toml_file = File::options().write(true).create(true).open(
                self.output
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join("Cargo.toml"),
            )?;
            cargo_toml_file.write_all(rendered_cargo_toml.as_bytes())?;
            let rendered_lib = self.tera.render("lib.templ", &Context::new())?;
            let mut cargo_lib_file = File::options()
                .write(true)
                .create(true)
                .open(self.output.parent().unwrap().join("lib.rs"))?;
            cargo_lib_file.write_all(rendered_lib.as_bytes())?;
        }
        Ok(())
    }

    /// Returns schema id from given directory entry path
    fn schema_id_from(schema_file: DirEntry) -> CodeGenResult<String> {
        let schema_file_path = schema_file.path();
        let schema_id = schema_file_path
            .file_name()
            .ok_or(IoError {
                source: Error::other(format!(
                    "Filename returned `None` for given path, the path terminates in ...: {}",
                    schema_file_path.display()
                )),
            })?
            .to_str()
            .ok_or(IoError {
                source: Error::other(format!(
                    "Given schema file path can not be converted valid UTF-8 string for extracting schema id: {}",
                    schema_file_path.display()
                )),
            })?;
        Ok(schema_id.to_string())
    }

    /// Generates code in Rust for given Ion Schema id with or without tests as per `test_input` value
    fn generate(&mut self, schema_id: String) -> CodeGenResult<()> {
        // load Ion schema with given schema id
        let schema = self.schema_system.load_isl_schema(schema_id)?;

        // this will be used for Rust to create mod.rs which lists all the generated modules
        let mut modules = vec![];
        let mut module_context = tera::Context::new();

        // Register a tera filter that can be used to convert a string based on case
        self.tera.register_filter("upper_camel", Self::upper_camel);
        self.tera.register_filter("snake", Self::snake);
        self.tera.register_filter("camel", Self::camel);

        // Register a tera filter that can be used to see if a type is built in data type or not
        self.tera
            .register_filter("is_built_in_type", Self::is_built_in_type);

        for isl_type in schema.types() {
            self.generate_abstract_data_type(&mut modules, isl_type)?;
        }

        if L::requires_modules() {
            self.generate_modules(&mut modules, &mut module_context)?;
        }
        Ok(())
    }

    // This method is only triggered for Rust code based on `L::requires_modules()`
    fn generate_modules(
        &mut self,
        modules: &mut Vec<String>,
        module_context: &mut Context,
    ) -> CodeGenResult<()> {
        module_context.insert("modules", &modules);
        let rendered = self.tera.render("mod.templ", module_context)?;
        let mut mod_file = File::options()
            .append(true)
            .create(true)
            .open(self.output.join("mod.rs"))?;
        mod_file.write_all(rendered.as_bytes())?;
        Ok(())
    }

    fn generate_abstract_data_type(
        &mut self,
        modules: &mut Vec<String>,
        isl_type: &IslType,
    ) -> CodeGenResult<()> {
        let isl_type_name = isl_type
            .name()
            .clone()
            .unwrap_or_else(|| format!("AnonymousType{}", self.anonymous_type_counter));

        let mut context = Context::new();
        let mut tera_fields = vec![];
        let mut imports: Vec<Import> = vec![];
        let mut code_gen_context = CodeGenContext::new();

        if self.is_root_type {
            context.insert("is_root_type", &true);
            self.is_root_type = false;
        } else {
            context.insert("is_root_type", &false);
        }

        // Set the ISL type name for the generated abstract data type
        context.insert("target_kind_name", &isl_type_name.to_case(Case::UpperCamel));

        let constraints = isl_type.constraints();
        for constraint in constraints {
            self.map_constraint_to_abstract_data_type(
                modules,
                &mut tera_fields,
                &mut imports,
                constraint,
                &mut code_gen_context,
            )?;
        }

        // add imports for the template
        context.insert("imports", &imports);

        // add fields for template
        // TODO: verify the `occurs` value within a field, by default the fields are optional.
        if let Some(abstract_data_type) = &code_gen_context.abstract_data_type {
            context.insert("fields", &tera_fields);
            context.insert("abstract_data_type", abstract_data_type);
        } else {
            return invalid_abstract_data_type_error(
                    "Can not determine abstract data type, constraints are mapping not mapping to an abstract data type.",
                );
        }

        if let Some(test_input) = &self.current_test_input {
            context.insert("generate_test", &true);
            let ion_string = fs::read_to_string(PathBuf::from(test_input))?;
            context.insert("ion_string", &ion_string);
        }
        self.render_generated_code(modules, &isl_type_name, &mut context, &mut code_gen_context)
    }

    fn render_generated_code(
        &mut self,
        modules: &mut Vec<String>,
        abstract_data_type_name: &str,
        context: &mut Context,
        code_gen_context: &mut CodeGenContext,
    ) -> CodeGenResult<()> {
        modules.push(L::file_name_for_type(abstract_data_type_name));

        // Render or generate file for the template with the given context
        let template: &Template = &code_gen_context.abstract_data_type.as_ref().try_into()?;
        let rendered = self
            .tera
            .render(&format!("{}.templ", L::template_name(template)), context)
            .unwrap();
        let mut file = File::create(self.output.join(format!(
            "{}.{}",
            L::file_name_for_type(abstract_data_type_name),
            L::file_extension()
        )))?;
        file.write_all(rendered.as_bytes())?;
        Ok(())
    }

    /// Provides name of the type reference that will be used for generated abstract data type
    fn type_reference_name(
        &mut self,
        isl_type_ref: &IslTypeRef,
        modules: &mut Vec<String>,
        imports: &mut Vec<Import>,
    ) -> CodeGenResult<String> {
        Ok(match isl_type_ref {
            IslTypeRef::Named(name, _) => {
                if !self.is_built_in_isl_type(name) {
                    imports.push(Import {
                        name: name.to_string(),
                    });
                }
                let schema_type: IonSchemaType = name.into();
                L::target_type(&schema_type)
            }
            IslTypeRef::TypeImport(_, _) => {
                unimplemented!("Imports in schema are not supported yet!");
            }
            IslTypeRef::Anonymous(type_def, _) => {
                self.anonymous_type_counter += 1;
                let name = format!("AnonymousType{}", self.anonymous_type_counter);
                self.generate_abstract_data_type(modules, type_def)?;
                imports.push(Import {
                    name: name.to_string(),
                });
                name
            }
        })
    }

    /// Maps the given constraint value to an abstract data type
    fn map_constraint_to_abstract_data_type(
        &mut self,
        modules: &mut Vec<String>,
        tera_fields: &mut Vec<Field>,
        imports: &mut Vec<Import>,
        constraint: &IslConstraint,
        code_gen_context: &mut CodeGenContext,
    ) -> CodeGenResult<()> {
        match constraint.constraint() {
            IslConstraintValue::Element(isl_type, _) => {
                let type_name = self.type_reference_name(isl_type, modules, imports)?;
                self.verify_abstract_data_type_consistency(
                    AbstractDataType::Sequence(type_name.to_owned()),
                    code_gen_context,
                )?;
                self.generate_struct_field(
                    tera_fields,
                    L::target_type_as_sequence(&type_name),
                    type_name,
                    "value",
                )?;
            }
            IslConstraintValue::Fields(fields, content_closed) => {
                // TODO: Check for `closed` annotation on fields and based on that return error while reading if there are extra fields.
                self.verify_abstract_data_type_consistency(
                    AbstractDataType::Struct(*content_closed),
                    code_gen_context,
                )?;
                for (name, value) in fields.iter() {
                    let type_name =
                        self.type_reference_name(value.type_reference(), modules, imports)?;

                    self.generate_struct_field(
                        tera_fields,
                        type_name,
                        value.type_reference().name(),
                        name,
                    )?;
                }
            }
            IslConstraintValue::Type(isl_type) => {
                let type_name = self.type_reference_name(isl_type, modules, imports)?;

                self.verify_abstract_data_type_consistency(
                    AbstractDataType::Value,
                    code_gen_context,
                )?;
                self.generate_struct_field(tera_fields, type_name, isl_type.name(), "value")?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Generates a struct field based on field name and value(data type)
    fn generate_struct_field(
        &mut self,
        tera_fields: &mut Vec<Field>,
        abstract_data_type_name: String,
        isl_type_name: String,
        field_name: &str,
    ) -> CodeGenResult<()> {
        tera_fields.push(Field {
            name: field_name.to_string(),
            isl_value: isl_type_name.to_string(),
            value: abstract_data_type_name,
        });
        Ok(())
    }

    /// Verify that the current abstract data type is same as previously determined abstract data type
    /// This is referring to abstract data type determined with each constraint that is verifies
    /// that all the constraints map to a single abstract data type and not different abstract data types.
    /// e.g.
    /// ```ion-schema
    /// type::{
    ///   name: foo,
    ///   type: string,
    ///   fields:{
    ///      source: String,
    ///      destination: String
    ///   }
    /// }
    /// ```
    /// For the above schema, both `fields` and `type` constraints map to different abstract data types
    /// respectively Struct(with given fields `source` and `destination`) and Value(with a single field that has String data type).
    fn verify_abstract_data_type_consistency(
        &mut self,
        current_abstract_data_type: AbstractDataType,
        code_gen_context: &mut CodeGenContext,
    ) -> CodeGenResult<()> {
        if let Some(abstract_data_type) = &code_gen_context.abstract_data_type {
            if abstract_data_type != &current_abstract_data_type {
                return invalid_abstract_data_type_error(format!("Can not determine abstract data type as current constraint {} conflicts with prior constraints for {}.", current_abstract_data_type, abstract_data_type));
            }
        } else {
            code_gen_context.with_abstract_data_type(current_abstract_data_type);
        }
        Ok(())
    }
}
