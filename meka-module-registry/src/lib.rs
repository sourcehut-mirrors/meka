//! Registry for mapping string paths to mlua loader function pointers.
//!
//! This crate exists to support `meka-config-evaluator` when using feature `mlua-module`. It
//! provides compile-time mapping from loader function paths (as strings) to function pointers,
//! allowing `meka-config-evaluator` subprocess to reconstruct `LoaderRegistry` from serialized
//! string data.

use meka_loader::{LoaderFn, LoaderRegistry};
use mlua_module_manifest::Manifest;
use phf::phf_map;
use std::borrow::Cow;
use std::collections::HashMap;
use std::result::Result;
use std::string::String;
use std::vec::Vec;

/// Perfect hash map for O(1) compile-time string-to-function lookups.
///
/// Add new entries here as new loader dependencies are added to Cargo.toml.
static LOADERS: phf::Map<&'static str, LoaderFn> = phf_map! {
    "fennel_src::loader" => fennel_src::loader,
    "meka::loader" => meka_loader::loader,
};

/// Convert list of (name, path) pairs to `LoaderRegistry`.
///
/// # Arguments
/// * `paths` - Vector of tuples where:
///   - First element: User-defined name for loader (what they'll `require()`)
///   - Second element: Function path string (e.g. "fennel_src::loader")
///
/// # Returns
/// * `Ok(LoaderRegistry)` - `HashMap` ready for use with `lua.add_function_searcher()`
/// * `Err(Vec<String>)` - List of unknown function paths which couldn't be resolved
///
/// # Example
/// ```no_run
/// let paths = vec![
///     ("fnl".to_string(), "fennel_src::loader".to_string()),
///     ("meka".to_string(), "meka::loader".to_string()),
/// ];
///
/// match meka_module_registry::build_loader_registry(paths) {
///     Ok(registry) => {
///         // Use with lua.add_function_searcher(registry)
///     }
///     Err(unknown) => {
///         eprintln!("Unknown loaders: {:?}", unknown);
///     }
/// }
/// ```
pub fn build_loader_registry(paths: Vec<(String, String)>) -> Result<LoaderRegistry, Vec<String>> {
    let mut registry = LoaderRegistry::with_capacity(paths.len());
    let mut unknown_paths = Vec::new();

    for (user_name, function_path) in paths {
        match LOADERS.get(function_path.as_str()) {
            Some(&loader_fn) => {
                // Map user's chosen name to resolved function pointer.
                registry.insert(Cow::from(user_name), loader_fn);
            }
            None => {
                // Track unknown paths for error reporting.
                unknown_paths.push(format!("{} -> {}", user_name, function_path));
            }
        }
    }

    if unknown_paths.is_empty() {
        Ok(registry)
    } else {
        Err(unknown_paths)
    }
}

/// Alternative API: Look up loader function by path.
///
/// # Arguments
/// * `path` - Function path string (e.g. "fennel_src::loader")
///
/// # Returns
/// * `Some(LoaderFn)` - Function pointer if found
/// * `None` - If path not recognized
#[inline]
pub fn lookup_loader(path: &str) -> Option<LoaderFn> {
    LOADERS.get(path).copied()
}

/// List all available loader paths.
///
/// Useful for debugging or generating documentation.
pub fn available_loaders() -> Vec<&'static str> {
    LOADERS.keys().copied().collect()
}
