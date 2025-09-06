use std::env;
use std::path::PathBuf;
use std::result::Result;
use std::string::String;
use std::vec::Vec;

/// Convert `syn::Path` containing multiple segments into `String` free of extraneous whitespace.
pub fn path_to_string(path: &syn::Path) -> String {
    let segments: Vec<String> = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect();
    segments.join("::")
}

/// Return the value of `$CARGO_MANIFEST_DIR` at runtime.
pub fn runtime_root() -> Result<PathBuf, env::VarError> {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    Ok(PathBuf::new().join(cargo_manifest_dir))
}
