pub(crate) const STRUCT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
    "/src/bin/ion/commands/beta/generate/templates/rust/struct.templ"
));
pub(crate) const RESULT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
"/src/bin/ion/commands/beta/generate/templates/rust/result.templ"
));
pub(crate) const NESTED_TYPE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
    "/src/bin/ion/commands/beta/generate/templates/rust/nested_type.templ"
));
pub(crate) const IMPORT: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
    "/src/bin/ion/commands/beta/generate/templates/rust/import.templ"
));