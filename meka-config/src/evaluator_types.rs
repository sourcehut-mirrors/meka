use mlua_module_manifest::Module;
use savefile_derive::Savefile;
use std::vec::Vec;

/// Input to meka-config-evaluator subprocess.
#[derive(Debug, Savefile)]
pub struct ConfigEvaluatorInput {
    pub module: Module,
    // (name, function_path)
    pub loader_paths: Vec<(String, String)>,
}
