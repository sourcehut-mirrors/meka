use fennel_compile::Compile;
use fennel_mount::Mount;
use meka_types::CatCowMap;
use mlua::{Function, Lua, MetaMethod, RegistryKey, Table, UserData, UserDataMethods, Value};
use mlua_searcher::AddSearcher as _;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

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
                    let globals = lua.globals();
                    globals.set("content", content.to_string())?;
                    let load = r#"local fennel = require("fennel")
                    return fennel.eval(content, {env = "_COMPILER"})"#;
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
                    let mut file = File::open(path)
                        .map_err(|e| mlua::Error::RuntimeError(format!("io error: {:#?}", e)))?;
                    file.read_to_string(&mut content)
                        .map_err(|e| mlua::Error::RuntimeError(format!("io error: {:#?}", e)))?;
                    let globals = lua.globals();
                    globals.set("content", content)?;
                    let load = r#"local fennel = require("fennel")
                    return fennel.eval(content, {env = "_COMPILER"})"#;
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
    modules: CatCowMap,
    globals: RegistryKey,
}

impl CatSearcher {
    fn new(modules: CatCowMap, globals: RegistryKey) -> Self {
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
            match this.modules.get(&name) {
                Some(content) => {
                    lua.mount_fennel().map_err(|e| {
                        mlua::Error::RuntimeError(format!("fennel-compile error: {:#?}", e))
                    })?;
                    let content = content
                        .cat()
                        .map_err(|e| mlua::Error::RuntimeError(format!("io error: {}", e)))?;
                    let content = lua.compile_fennel_string(&content).map_err(|e| {
                        mlua::Error::RuntimeError(format!("fennel-compile error: {:#?}", e))
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
    modules: CatCowMap,
    globals: RegistryKey,
}

impl MacroCatSearcher {
    fn new(modules: CatCowMap, globals: RegistryKey) -> Self {
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
            match this.modules.get(&name) {
                Some(content) => {
                    lua.mount_fennel().map_err(|e| {
                        mlua::Error::RuntimeError(format!("fennel-compile error: {:#?}", e))
                    })?;
                    let content = content
                        .cat()
                        .map_err(|e| mlua::Error::RuntimeError(format!("io error: {}", e)))?;
                    let globals = lua.globals();
                    globals.set("content", content.to_string())?;
                    let load = r#"return require("fennel").eval(content, {env = "_COMPILER"})"#;
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

/// Extend `mlua::Lua` to support `require`ing Fennel modules and importing Fennel macros
/// by name.
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
    fn add_cat_searcher_fnl(&self, modules: CatCowMap) -> Result<()>;

    /// Like `add_cat_searcher_fnl`, but for modules containing Fennel macros.
    fn add_cat_searcher_fnl_macros(&self, modules: CatCowMap) -> Result<()>;
}

impl AddSearcher for Lua {
    fn add_searcher_fnl_macros(
        &self,
        modules: HashMap<Cow<'static, str>, Cow<'static, str>>,
    ) -> Result<()> {
        let globals: Table = self.globals();
        let fennel: Table = self.load(r#"return require("fennel")"#).eval()?;
        let macro_searchers: Table = fennel.get("macro-searchers")?;
        let registry_key = self.create_registry_value(globals)?;
        let macro_searcher = MacroSearcher::new(modules, registry_key);
        macro_searchers
            .set(macro_searchers.len()? + 1, macro_searcher)
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
                        lua.mount_fennel().map_err(|e| {
                            mlua::Error::RuntimeError(format!("fennel-compile error: {:#?}", e))
                        })?;
                        let content = lua.compile_fennel_file(path).map_err(|e| {
                            mlua::Error::RuntimeError(format!("fennel-compile error: {:#?}", e))
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
        let fennel: Table = self.load(r#"return require("fennel")"#).eval()?;
        let macro_searchers: Table = fennel.get("macro-searchers")?;
        let registry_key = self.create_registry_value(globals)?;
        let macro_searcher = MacroPathSearcher::new(modules, registry_key);
        macro_searchers
            .set(macro_searchers.len()? + 1, macro_searcher)
            .map_err(|e| e.into())
    }

    fn add_cat_searcher_fnl(&self, modules: CatCowMap) -> Result<()> {
        let globals = self.globals();
        let searchers: Table = globals.get::<Table>("package")?.get("searchers")?;
        let registry_key = self.create_registry_value(globals)?;
        let searcher = CatSearcher::new(modules, registry_key);
        searchers
            .set(searchers.len()? + 1, searcher)
            .map_err(|e| e.into())
    }

    fn add_cat_searcher_fnl_macros(&self, modules: CatCowMap) -> Result<()> {
        self.mount_fennel()
            .map_err(|e| mlua::Error::RuntimeError(format!("fennel-compile error: {:#?}", e)))?;
        let globals: Table = self.globals();
        let fennel: Table = self.load(r#"return require("fennel")"#).eval()?;
        let macro_searchers: Table = fennel.get("macro-searchers")?;
        let registry_key = self.create_registry_value(globals)?;
        let macro_searcher = MacroCatSearcher::new(modules, registry_key);
        macro_searchers
            .set(macro_searchers.len()? + 1, macro_searcher)
            .map_err(|e| e.into())
    }
}
