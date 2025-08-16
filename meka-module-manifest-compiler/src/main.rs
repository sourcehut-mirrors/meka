use meka_module_manifest::{CompiledNamedTextManifest, CompiledNamedTextManifestInitError};
use mlua_module_manifest::NamedTextManifest;
use savefile::{CURRENT_SAVEFILE_LIB_VERSION, load_from_mem, save_to_mem};
use std::convert::TryFrom;
use std::io;
use std::io::{Read, Write};
use std::vec::Vec;

const IO_STDIN_READ_TO_END_EXPECT: &str = "Failed to read from stdin";
const IO_STDOUT_WRITEALL_EXPECT: &str = "Failed to write result";
const SAVEFILE_LOAD_FROM_MEM_EXPECT: &str = "Failed to deserialize manifest";
const SAVEFILE_SAVE_TO_MEM_EXPECT: &str = "Failed to serialize result";

fn main() {
    // Read serialized manifest from stdin.
    let mut buffer = Vec::new();
    io::stdin()
        .read_to_end(&mut buffer)
        .expect(IO_STDIN_READ_TO_END_EXPECT);

    // Deserialize manifest.
    let manifest: NamedTextManifest = load_from_mem(&buffer, CURRENT_SAVEFILE_LIB_VERSION.into())
        .expect(SAVEFILE_LOAD_FROM_MEM_EXPECT);

    // Use public `TryFrom` API.
    let result = CompiledNamedTextManifest::try_from(manifest);

    // Serialize result back.
    let serialized = save_to_mem(CURRENT_SAVEFILE_LIB_VERSION.into(), &result)
        .expect(SAVEFILE_SAVE_TO_MEM_EXPECT);

    // Write serialized result to stdout.
    io::stdout()
        .write_all(&serialized)
        .expect(IO_STDOUT_WRITEALL_EXPECT);
}
