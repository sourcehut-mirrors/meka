use fennel_compile::Compile;
use io_cat::Cat;
use meka_types::CatCow;
use mlua::{Function, Lua, MetaMethod, RegistryKey, Table, UserData, UserDataMethods, Value};
use mlua_searcher::AddSearcher as _;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::error::Error;
use crate::types::Result;

/// Stores Fennel macro modules indexed by module name, and provides an `mlua::MetaMethod`
/// to enable importing the stored macros by name in an `mlua::Lua`.
struct MacroSearcher {
    /// A `HashMap` of Fennel macro modules in string representation, indexed by module name.
    modules: HashMap<Cow<'static, str>, Cow<'static, str>>,

    /// An `mlua::RegistryKey` whose value is the Lua environment within which the user made
    /// the request to instantiate a `MacroSearcher` for `modules`.
    globals: RegistryKey,
}

impl MacroSearcher {
    fn new(modules: HashMap<Cow<'static, str>, Cow<'static, str>>, globals: RegistryKey) -> Self {
        Self { modules, globals }
    }
}

impl UserData for MacroSearcher {
    fn add_methods<M>(methods: &mut M)
    where
        M: UserDataMethods<Self>,
    {
        methods.add_meta_method(MetaMethod::Call, |lua, this, name: String| {
            let name = Cow::from(name);
            match this.modules.get(&name) {
                Some(content) => {
                    let content = match content {
                        Cow::Borrowed(content) => content,
                        Cow::Owned(content) => content.as_str(),
                    };
                    let fennel = mlua_utils::require::<Table>(lua, "fennel").map_err(|e| {
                        mlua::Error::RuntimeError(format!("fennel-searcher error: {}", e))
                    })?;
                    let globals = lua.globals();
                    globals.set("content", content.to_string())?;
                    globals.set("fennel", fennel)?;
                    let load = r#"return fennel.eval(content, {env = "_COMPILER"})"#;
                    Ok(Value::Function(
                        lua.load(load)
                            .set_name(name.as_ref())
                            .set_environment(lua.registry_value::<Table>(&this.globals)?)
                            .into_function()?,
                    ))
                }
                None => Ok(Value::Nil),
            }
        });
    }
}

/// Like `MacroSearcher`, but with `modules` values given as paths to Fennel macro modules.
///
/// Facilitates Fennel macro module reloading.
struct MacroPathSearcher<P>
where
    P: 'static + AsRef<Path> + Send,
{
    modules: HashMap<Cow<'static, str>, P>,
    globals: RegistryKey,
}

impl<P> MacroPathSearcher<P>
where
    P: 'static + AsRef<Path> + Send,
{
    fn new(modules: HashMap<Cow<'static, str>, P>, globals: RegistryKey) -> Self {
        Self { modules, globals }
    }
}

impl<P> UserData for MacroPathSearcher<P>
where
    P: 'static + AsRef<Path> + Send,
{
    fn add_methods<M>(methods: &mut M)
    where
        M: UserDataMethods<Self>,
    {
        methods.add_meta_method(MetaMethod::Call, |lua, this, name: String| {
            let name = Cow::from(name);
            match this.modules.get(&name) {
                Some(ref path) => {
                    let path = path.as_ref();
                    let mut content = String::new();
                    let mut file = File::open(path).map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "fennel-searcher error: io error: {:?}",
                            e
                        ))
                    })?;
                    file.read_to_string(&mut content).map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "fennel-searcher error: io error: {:?}",
                            e
                        ))
                    })?;
                    let fennel = mlua_utils::require::<Table>(lua, "fennel").map_err(|e| {
                        mlua::Error::RuntimeError(format!("fennel-searcher error: {}", e))
                    })?;
                    let globals = lua.globals();
                    globals.set("content", content)?;
                    globals.set("fennel", fennel)?;
                    let load = r#"return fennel.eval(content, {env = "_COMPILER"})"#;
                    Ok(Value::Function(
                        lua.load(load)
                            .set_name(name.as_ref())
                            .set_environment(lua.registry_value::<Table>(&this.globals)?)
                            .into_function()?,
                    ))
                }
                None => Ok(Value::Nil),
            }
        });
    }
}

/// Like `MacroSearcher`, but with `modules` values given as either strings containing or
/// paths to Fennel modules.
struct CatSearcher {
    modules: CatCow,
    globals: RegistryKey,
}

impl CatSearcher {
    fn new(modules: CatCow, globals: RegistryKey) -> Self {
        Self { modules, globals }
    }
}

impl UserData for CatSearcher {
    fn add_methods<M>(methods: &mut M)
    where
        M: UserDataMethods<Self>,
    {
        methods.add_meta_method(MetaMethod::Call, |lua, this, name: String| {
            let name = Cow::from(name);
            match this.modules.0.get(&name) {
                Some(content) => {
                    let content = content.cat().map_err(|e| {
                        mlua::Error::RuntimeError(format!("fennel-searcher error: io error: {}", e))
                    })?;
                    let content = lua.compile_fennel_string(&content).map_err(|e| {
                        mlua::Error::RuntimeError(format!("fennel-searcher error: {:?}", e))
                    })?;
                    Ok(Value::Function(
                        lua.load(&content)
                            .set_name(name.as_ref())
                            .set_environment(lua.registry_value::<Table>(&this.globals)?)
                            .into_function()?,
                    ))
                }
                None => Ok(Value::Nil),
            }
        });
    }
}

/// Like `CatSearcher`, but for modules containing Fennel macros.
struct MacroCatSearcher {
    modules: CatCow,
    globals: RegistryKey,
}

impl MacroCatSearcher {
    fn new(modules: CatCow, globals: RegistryKey) -> Self {
        Self { modules, globals }
    }
}

impl UserData for MacroCatSearcher {
    fn add_methods<M>(methods: &mut M)
    where
        M: UserDataMethods<Self>,
    {
        methods.add_meta_method(MetaMethod::Call, |lua, this, name: String| {
            let name = Cow::from(name);
            match this.modules.0.get(&name) {
                Some(content) => {
                    let content = content.cat().map_err(|e| {
                        mlua::Error::RuntimeError(format!("fennel-searcher error: io error: {}", e))
                    })?;
                    let fennel = mlua_utils::require::<Table>(lua, "fennel").map_err(|e| {
                        mlua::Error::RuntimeError(format!("fennel-searcher error: {}", e))
                    })?;
                    let globals = lua.globals();
                    globals.set("content", content.to_string())?;
                    globals.set("fennel", fennel)?;
                    let load = r#"return fennel.eval(content, {env = "_COMPILER"})"#;
                    Ok(Value::Function(
                        lua.load(load)
                            .set_name(name.as_ref())
                            .set_environment(lua.registry_value::<Table>(&this.globals)?)
                            .into_function()?,
                    ))
                }
                None => Ok(Value::Nil),
            }
        });
    }
}

/// Extend `mlua::Lua` to support `require`ing Fennel modules and importing Fennel macros by name.
pub trait AddSearcher {
    /// Add a `HashMap` of Fennel macro modules indexed by module name to Fennel's
    /// `fennel.macro-searchers` table in an `mlua::Lua`, with lookup functionality
    /// provided by the `fennel_searcher::MacroSearcher` struct.
    fn add_searcher_fnl_macros(
        &self,
        modules: HashMap<Cow<'static, str>, Cow<'static, str>>,
    ) -> Result<()>;

    /// Add a `HashMap` of Fennel module paths indexed by module name to Lua's
    /// `package.searchers` table in an `mlua::Lua`, with lookup functionality
    /// provided by the `mlua_searcher::PolySearcher` struct, and Fennel-to-Lua
    /// compilation done on-the-fly with `fennel-compile`.
    ///
    /// Facilitates Fennel module reloading.
    fn add_path_searcher_fnl<P>(&self, modules: HashMap<Cow<'static, str>, P>) -> Result<()>
    where
        P: 'static + AsRef<Path> + Send;

    /// Like `add_searcher_fnl_macros`, except reads the file at path given in `modules`
    /// to string at runtime to facilitate Fennel macro module reloading.
    fn add_path_searcher_fnl_macros<P>(&self, modules: HashMap<Cow<'static, str>, P>) -> Result<()>
    where
        P: 'static + AsRef<Path> + Send;

    /// Like `add_path_searcher_fnl`, but accepts heterogenous strings and paths - assumed to
    /// contain Fennel text directly and by resolution, respectively - indexed by module name.
    fn add_cat_searcher_fnl(&self, modules: CatCow) -> Result<()>;

    /// Like `add_cat_searcher_fnl`, but for modules containing Fennel macros.
    fn add_cat_searcher_fnl_macros(&self, modules: CatCow) -> Result<()>;
}

impl AddSearcher for Lua {
    fn add_searcher_fnl_macros(
        &self,
        modules: HashMap<Cow<'static, str>, Cow<'static, str>>,
    ) -> Result<()> {
        let globals: Table = self.globals();
        let fennel = mlua_utils::require::<Table>(self, "fennel")
            .map_err(|e| Error::FailedToImportFennel(e))?;
        let macro_searchers: Table = fennel.get("macro-searchers")?;
        let registry_key = self.create_registry_value(globals)?;
        let macro_searcher = MacroSearcher::new(modules, registry_key);
        macro_searchers
            .raw_insert(1, macro_searcher)
            .map_err(|e| e.into())
    }

    fn add_path_searcher_fnl<P>(&self, modules: HashMap<Cow<'static, str>, P>) -> Result<()>
    where
        P: 'static + AsRef<Path> + Send,
    {
        let modules: HashMap<
            Cow<'static, str>,
            Box<dyn Fn(&Lua, Table, &str) -> mlua::Result<Function> + Send>,
        > = modules
            .into_iter()
            .map(|(n, p)| {
                let loader: Box<dyn Fn(&Lua, Table, &str) -> mlua::Result<Function> + Send> =
                    Box::new(move |lua, env, name| {
                        let path = p.as_ref();
                        let content = lua.compile_fennel_file(path).map_err(|e| {
                            mlua::Error::RuntimeError(format!("fennel-searcher error: {:?}", e))
                        })?;
                        Ok(lua
                            .load(&content)
                            .set_name(name)
                            .set_environment(env)
                            .into_function()?)
                    });
                (n, loader)
            })
            .collect();

        self.add_closure_searcher(modules).map_err(|e| e.into())
    }

    fn add_path_searcher_fnl_macros<P>(&self, modules: HashMap<Cow<'static, str>, P>) -> Result<()>
    where
        P: 'static + AsRef<Path> + Send,
    {
        let globals: Table = self.globals();
        let fennel = mlua_utils::require::<Table>(self, "fennel")
            .map_err(|e| Error::FailedToImportFennel(e))?;
        let macro_searchers: Table = fennel.get("macro-searchers")?;
        let registry_key = self.create_registry_value(globals)?;
        let macro_searcher = MacroPathSearcher::new(modules, registry_key);
        macro_searchers
            .raw_insert(1, macro_searcher)
            .map_err(|e| e.into())
    }

    fn add_cat_searcher_fnl(&self, modules: CatCow) -> Result<()> {
        let globals = self.globals();
        let searchers: Table = globals.get::<Table>("package")?.get("searchers")?;
        let registry_key = self.create_registry_value(globals)?;
        let searcher = CatSearcher::new(modules, registry_key);
        searchers.raw_insert(2, searcher).map_err(|e| e.into())
    }

    fn add_cat_searcher_fnl_macros(&self, modules: CatCow) -> Result<()> {
        let globals: Table = self.globals();
        let fennel = mlua_utils::require::<Table>(self, "fennel")
            .map_err(|e| Error::FailedToImportFennel(e))?;
        let macro_searchers: Table = fennel.get("macro-searchers")?;
        let registry_key = self.create_registry_value(globals)?;
        let macro_searcher = MacroCatSearcher::new(modules, registry_key);
        macro_searchers
            .raw_insert(1, macro_searcher)
            .map_err(|e| e.into())
    }
}
