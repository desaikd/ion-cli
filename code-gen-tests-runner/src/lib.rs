use convert_case::{Case, Casing};
use glob::{glob, Paths};
use proc_macro::Span;

use quote::quote;
use std::fs;

use proc_macro2::TokenStream;
use std::path::{Path, PathBuf};

use syn::Ident;

// Form canonical name without any punctuation/delimiter or special character
fn canonical_fn_name(s: &str) -> String {
    // remove delimiters and special characters
    s.replace(
        &['"', ' ', '.', ':', '-', '*', '/', '\\', '\n', '\t', '\r'][..],
        "_",
    )
}

/// Return the concatenation of two token-streams
fn concat_ts_cnt(
    accu: (u64, proc_macro2::TokenStream),
    other: proc_macro2::TokenStream,
) -> (u64, proc_macro2::TokenStream) {
    let (accu_cnt, accu_ts) = accu;
    if accu_cnt == 0 {
        return (
            accu_cnt + 1,
            quote! {
                #[allow(clippy::all)]
                mod ion_data_model;
                use ion_rs::ReaderBuilder;
                use ion_rs::TextWriterBuilder;
                use ion_rs::IonWriter;
                use ion_rs::Element;
                use ion_rs::IonResult;
            #accu_ts #other },
        );
    }
    (accu_cnt + 1, quote! { #accu_ts #other })
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

#[proc_macro]
pub fn test_roundtrip_generated_code(_attr: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let func_copy: proc_macro2::TokenStream = TokenStream::default();
    let paths: Paths = glob(&format!(
        "{}/code-gen-tests-runner/tests/schema/*",
        workspace_dir().as_os_str().to_str().unwrap()
    ))
    .expect("No such file or directory");

    // for each path generate a test-function and fold them to single tokenstream
    let result = paths
        .map(|path| {
            let path_as_str = path.expect("No such file or directory")
                .into_os_string()
                .into_string()
                .expect("bad encoding");
            println!("{path_as_str}");

            let binding = PathBuf::from(path_as_str.clone());
            let module_name_string = binding.file_stem().expect("There is no file in this directory").to_str().unwrap();
            let module_name = Ident::new(module_name_string, Span::call_site().into());

            let _schema_string = fs::read_to_string(PathBuf::from(path_as_str.clone())).expect("No such schema file");
            let class_name = Ident::new(&module_name_string.to_case(Case::Pascal), Span::call_site().into());
            let test_name = format!("test_roundtrip_generated_code_{}", &module_name_string);

            let ion_string = fs::read_to_string(PathBuf::from(path_as_str.replace("schema", "input").replace("isl", "ion"))).unwrap_or_else(|_| panic!("No such ion file {}", path_as_str));

            // create function name without any delimiter or special character
            let test_name = canonical_fn_name(&test_name);

            // quote! requires proc_macro2 elements
            let test_ident = proc_macro2::Ident::new(&test_name, proc_macro2::Span::call_site());

            // Generate test for checking if the roundtrip read and write API of generated code results into equal Ion values
            quote! {
                use crate::ion_data_model::#module_name::#class_name;

                #[test]
                #[allow(non_snake_case)]
                fn # test_ident () -> IonResult<()> {
                         let mut reader = ReaderBuilder::new().build(#ion_string)?;
                         let mut buffer = Vec::new();
                         let mut text_writer = TextWriterBuilder::default().build(&mut buffer)?;
                         // read given Ion value using Ion reader
                         let mut x: #class_name = #class_name::read_from(&mut reader)?;
                         // write the generated abstract data type using Ion writer
                         x.write_to(&mut text_writer)?;
                         text_writer.flush()?;
                         // compare given Ion value with round tripped Ion value written using abstract data type's `write_to` API
                         assert_eq!(Element::read_one(text_writer.output().as_slice())?, (Element::read_one(#ion_string)?));

                         Ok(())
                }
            }
        }).fold((0, func_copy), concat_ts_cnt);

    // panic, the pattern did not match any file or folder
    if result.0 == 0 {
        panic!("no resource matching the pattern");
    }
    // transforming proc_macro2::TokenStream into proc_macro::TokenStream
    result.1.into()
}
