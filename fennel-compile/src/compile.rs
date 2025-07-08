use mlua::{Function, Lua, Table, Value};
use std::io::Read;
use std::path::Path;

use crate::error::Error;
use crate::types::Result;

pub trait Compile {
    /// Compile Fennel bytes to Lua. Assumes Fennel is available in Lua's
    /// `package.searchers` table in the `mlua::Lua`.
    fn compile_fennel_bytes(&self, fnl_bytes: &[u8]) -> Result<String>;

    /// Compile Fennel string to Lua. Assumes Fennel is available in Lua's
    /// `package.searchers` table in the `mlua::Lua`.
    fn compile_fennel_string(&self, fnl_str: &str) -> Result<String>;

    /// Compile Fennel file to Lua. Assumes Fennel is available in Lua's
    /// `package.searchers` table in the `mlua::Lua`.
    fn compile_fennel_file<P>(&self, fnl_path: P) -> Result<String>
    where
        P: AsRef<Path>;
}

impl Compile for Lua {
    fn compile_fennel_bytes(&self, fnl_bytes: &[u8]) -> Result<String> {
        self.compile_fennel_string(std::str::from_utf8(fnl_bytes)?)
    }

    fn compile_fennel_string(&self, fnl_str: &str) -> Result<String> {
        let fennel = mlua_utils::require::<Table>(self, "fennel")
            .map_err(|e| Error::FailedToImportFennel(e))?;
        let compile_string: Value = fennel.get::<Value>("compileString")?;
        let compile_string: Function = match compile_string {
            Value::Function(f) => f,
            _ => return Err(Error::MissingFennelCompileStringFunction),
        };
        let s = self.create_string(fnl_str)?;
        compile_string.call::<String>(s).map_err(|e| e.into())
    }

    fn compile_fennel_file<P>(&self, fnl_path: P) -> Result<String>
    where
        P: AsRef<Path>,
    {
        let mut fnl_string = String::new();
        let mut file = std::fs::File::open(fnl_path)?;
        file.read_to_string(&mut fnl_string)?;
        self.compile_fennel_string(&fnl_string)
    }
}
