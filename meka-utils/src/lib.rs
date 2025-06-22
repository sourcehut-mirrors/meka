use std::env;
use std::path::PathBuf;

/// Return the value of `$CARGO_MANIFEST_DIR` at runtime.
pub fn runtime_root() -> Result<PathBuf, env::VarError> {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    Ok(PathBuf::new().join(cargo_manifest_dir))
}
