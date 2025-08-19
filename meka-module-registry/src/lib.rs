//! Registry for mapping string paths to mlua loader function pointers.
//!
//! This crate exists to support `meka-config-evaluator` when using the `mlua-module`
//! feature. It provides a compile-time mapping from loader function paths (as strings)
//! to actual function pointers, allowing the evaluator subprocess to reconstruct a
//! `LoaderRegistry` from serialized string data.

use mlua::{Function, Lua, Table};
use mlua_module_manifest::Manifest;
use phf::phf_map;
use std::borrow::Cow;
use std::collections::HashMap;

/// Type alias for mlua loader function signature.
pub type LoaderFn = fn(&Lua, Table, &str) -> mlua::Result<Function>;

/// Type alias for the loader registry used by mlua's add_function_searcher.
pub type LoaderRegistry = HashMap<Cow<'static, str>, LoaderFn>;

/// Perfect hash map for O(1) compile-time string-to-function lookups.
/// Add new entries here as new loader dependencies are added to Cargo.toml.
static LOADERS: phf::Map<&'static str, LoaderFn> = phf_map! {
    "fennel_src::loader" => fennel_src::loader,
    "meka::loader" => meka_loader,
};

/// Convert a list of (name, path) pairs to a LoaderRegistry.
///
/// # Arguments
/// * `paths` - Vector of tuples where:
///   - First element: User-defined name for the loader (what they'll `require()`)
///   - Second element: Function path string (e.g., "fennel_src::loader")
///
/// # Returns
/// * `Ok(LoaderRegistry)` - HashMap ready for use with `lua.add_function_searcher()`
/// * `Err(Vec<String>)` - List of unknown function paths that couldn't be resolved
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
                // Map user's chosen name to the resolved function pointer
                registry.insert(Cow::from(user_name), loader_fn);
            }
            None => {
                // Track unknown paths for error reporting
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

/// Alternative API: Look up a single loader function by path.
///
/// # Arguments
/// * `path` - Function path string (e.g., "fennel_src::loader")
///
/// # Returns
/// * `Some(LoaderFn)` - The function pointer if found
/// * `None` - If the path is not recognized
#[inline]
pub fn lookup_loader(path: &str) -> Option<LoaderFn> {
    LOADERS.get(path).copied()
}

/// Get a list of all available loader paths.
/// Useful for debugging or generating documentation.
pub fn available_loaders() -> Vec<&'static str> {
    LOADERS.keys().copied().collect()
}

/// Implementation of the meka loader function.
/// This provides the `meka.manifest` module within Lua configs.
fn meka_loader(lua: &Lua, env: Table, name: &str) -> mlua::Result<Function> {
    // Get or create globals
    let globals = lua.globals();

    // Create the meka table
    let meka_table = lua.create_table()?;

    // Get the Manifest loader and call it to get the manifest module
    let manifest_loader: Function = Manifest::loader(lua, env.clone(), "manifest")?;
    let manifest_module: Table = manifest_loader.call(())?;

    // Set manifest in the meka table
    meka_table.set("manifest", manifest_module)?;

    // Set the meka table in globals (so it's accessible as a global)
    globals.set("meka", meka_table.clone())?;

    // Return a function that returns the meka table
    lua.load("return meka")
        .set_name(name)
        .set_environment(env)
        .into_function()
}
