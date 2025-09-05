use meka_config::evaluator_types::ConfigEvaluatorInput;
use meka_config::{Config, ConfigInitError};
use meka_loader::LoaderRegistry;
use meka_module_registry::build_loader_registry;
use mlua_module_manifest::Manifest;
use savefile::{CURRENT_SAVEFILE_LIB_VERSION, load_from_mem, save_to_mem};
use std::io;
use std::io::{Read, Write};
use std::vec::Vec;

const IO_STDIN_READ_TO_END_EXPECT: &str = "Failed to read from stdin";
const IO_STDOUT_WRITEALL_EXPECT: &str = "Failed to write result";
const SAVEFILE_LOAD_FROM_MEM_EXPECT: &str = "Failed to deserialize input";
const SAVEFILE_SAVE_TO_MEM_EXPECT: &str = "Failed to serialize result";

fn main() {
    // Read serialized input from stdin.
    let mut buffer = Vec::new();
    io::stdin()
        .read_to_end(&mut buffer)
        .expect(IO_STDIN_READ_TO_END_EXPECT);

    // Deserialize input.
    let ConfigEvaluatorInput {
        module,
        loader_paths,
    } = load_from_mem(&buffer, CURRENT_SAVEFILE_LIB_VERSION.into())
        .expect(SAVEFILE_LOAD_FROM_MEM_EXPECT);

    // Evaluate config and get result.
    let result: Result<Config, ConfigInitError> = match build_loader_registry(loader_paths) {
        Ok(lreg) => {
            let lreg: Option<LoaderRegistry> = Some(lreg);
            Config::new(module, lreg)
        }
        Err(unknown) => Err(ConfigInitError::ConfigEvaluator(format!(
            "Unknown loader paths: {:?}",
            unknown
        ))),
    };

    // Serialize result.
    let serialized = save_to_mem(CURRENT_SAVEFILE_LIB_VERSION.into(), &result)
        .expect(SAVEFILE_SAVE_TO_MEM_EXPECT);

    // Write serialized result to stdout.
    io::stdout()
        .write_all(&serialized)
        .expect(IO_STDOUT_WRITEALL_EXPECT);
}
