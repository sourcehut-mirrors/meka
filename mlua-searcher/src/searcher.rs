use meka_types::CatMap;
use mlua::{Function, Lua, MetaMethod, RegistryKey, Table, UserData, UserDataMethods, Value};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::types::Result;

/// Stores Lua modules indexed by module name, and provides an `mlua::MetaMethod` to
/// enable `require`ing the stored modules by name in an `mlua::Lua`.
struct Searcher {
    /// A `HashMap` of Lua modules in string representation, indexed by module name.
    ///
    /// Uses `Cow<'static, str>` types to allow both `&'static str` and owned `String`.
    modules: HashMap<Cow<'static, str>, Cow<'static, str>>,

    /// An `mlua::RegistryKey` whose value is the Lua environment within which the user
    /// made the request to instantiate a `Searcher` for `modules`.
    globals: RegistryKey,
}

impl Searcher {
    fn new(modules: HashMap<Cow<'static, str>, Cow<'static, str>>, globals: RegistryKey) -> Self {
        Self { modules, globals }
    }
}

impl UserData for Searcher {
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
                    let env = lua.registry_value::<Table>(&this.globals)?;
                    Ok(Value::Function(
                        lua.load(content)
                            .set_name(name.as_ref())
                            .set_environment(env)
                            .into_function()?,
                    ))
                }
                None => Ok(Value::Nil),
            }
        });
    }
}

/// Like `Searcher`, but with `modules` values given as paths to files the content of
/// which can be read as Lua source code.
///
/// Facilitates Lua module reloading, and module reloading of any other programming
/// language whose source code can be compiled to Lua.
struct PathSearcherPoly<P>
where
    P: 'static + AsRef<Path> + Send,
{
    modules: HashMap<Cow<'static, str>, P>,
    globals: RegistryKey,

    /// Function to read file content as Lua source code.
    transform: Box<dyn Fn(PathBuf) -> mlua::Result<String> + Send>,
}

impl<P> PathSearcherPoly<P>
where
    P: 'static + AsRef<Path> + Send,
{
    fn new(
        modules: HashMap<Cow<'static, str>, P>,
        globals: RegistryKey,
        transform: Box<dyn Fn(PathBuf) -> mlua::Result<String> + Send>,
    ) -> Self {
        Self {
            modules,
            globals,
            transform,
        }
    }
}

impl<P> UserData for PathSearcherPoly<P>
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
                    let path = path.as_ref().to_path_buf();
                    let content = (this.transform)(path)?;
                    let env = lua.registry_value::<Table>(&this.globals)?;
                    Ok(Value::Function(
                        lua.load(&content)
                            .set_name(name.as_ref())
                            .set_environment(env)
                            .into_function()?,
                    ))
                }
                None => Ok(Value::Nil),
            }
        });
    }
}

/// Like `Searcher`, but with closures as `modules` values, to facilitate setting up an
/// `mlua::Lua` with Rust code.
///
/// Enables exposing `UserData` types to an `mlua::Lua`.
struct ClosureSearcher {
    /// Closures must accept three parameters:
    ///
    /// 1. An `&mlua::Lua`, which the closure can do what it wants with.
    ///
    /// 2. An `mlua::Table` containing globals (i.e. Lua's `_G`), which can be passed to
    ///    `Chunk.set_environment()`.
    ///
    /// 3. The name of the module to be loaded (`&str`).
    ///
    /// Closures must return an `mlua::Result`-wrapped `Function`. This `Function` acts as
    /// the module loader.
    modules:
        HashMap<Cow<'static, str>, Box<dyn Fn(&Lua, Table, &str) -> mlua::Result<Function> + Send>>,

    globals: RegistryKey,
}

impl ClosureSearcher {
    fn new(
        modules: HashMap<
            Cow<'static, str>,
            Box<dyn Fn(&Lua, Table, &str) -> mlua::Result<Function> + Send>,
        >,
        globals: RegistryKey,
    ) -> Self {
        Self { modules, globals }
    }
}

impl UserData for ClosureSearcher {
    fn add_methods<M>(methods: &mut M)
    where
        M: UserDataMethods<Self>,
    {
        methods.add_meta_method(MetaMethod::Call, |lua: &Lua, this, name: String| {
            let name = Cow::from(name);
            match this.modules.get(&name) {
                Some(ref closure) => Ok(Value::Function(closure(
                    lua,
                    lua.registry_value::<Table>(&this.globals)?,
                    name.as_ref(),
                )?)),
                None => Ok(Value::Nil),
            }
        });
    }
}

/// Like `Searcher`, but with function pointers as `modules` values, to facilitate setting
/// up an `mlua::Lua` with Rust code.
///
/// Enables exposing `UserData` types to an `mlua::Lua`.
struct FunctionSearcher {
    /// Functions must accept three parameters:
    ///
    /// 1. An `&mlua::Lua`, which the function body can do what it wants with.
    ///
    /// 2. An `mlua::Table` containing globals (i.e. Lua's `_G`), which can be passed to
    ///    `Chunk.set_environment()`.
    ///
    /// 3. The name of the module to be loaded (`&str`).
    ///
    /// Functions must return an `mlua::Result`-wrapped `Function`. This `Function` acts
    /// as the module loader.
    modules: HashMap<Cow<'static, str>, fn(&Lua, Table, &str) -> mlua::Result<Function>>,

    globals: RegistryKey,
}

impl FunctionSearcher {
    fn new(
        modules: HashMap<Cow<'static, str>, fn(&Lua, Table, &str) -> mlua::Result<Function>>,
        globals: RegistryKey,
    ) -> Self {
        Self { modules, globals }
    }
}

impl UserData for FunctionSearcher {
    fn add_methods<M>(methods: &mut M)
    where
        M: UserDataMethods<Self>,
    {
        methods.add_meta_method(MetaMethod::Call, |lua: &Lua, this, name: String| {
            let name = Cow::from(name);
            match this.modules.get(&name) {
                Some(ref function) => Ok(Value::Function(function(
                    lua,
                    lua.registry_value::<Table>(&this.globals)?,
                    name.as_ref(),
                )?)),
                None => Ok(Value::Nil),
            }
        });
    }
}

/// Like `Searcher`, but with `CatMap` to facilitate indexing heterogenous strings and paths -
/// all presumed to resolve to Lua module content - by module names in `modules`.
struct CatSearcher {
    modules: CatMap,
    globals: RegistryKey,
}

impl CatSearcher {
    fn new(modules: CatMap, globals: RegistryKey) -> Self {
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
                    let content = content
                        .cat()
                        .map_err(|e| mlua::Error::RuntimeError(format!("io error: {}", e)))?;
                    let env = lua.registry_value::<Table>(&this.globals)?;
                    Ok(Value::Function(
                        lua.load(&content)
                            .set_name(name.as_ref())
                            .set_environment(env)
                            .into_function()?,
                    ))
                }
                None => Ok(Value::Nil),
            }
        });
    }
}

/// Extend `mlua::Lua` to support `require`ing Lua modules by name.
pub trait AddSearcher {
    /// Add a `HashMap` of Lua modules indexed by module name to Lua's `package.searchers`
    /// table in an `mlua::Lua`, with lookup functionality provided by the
    /// `mlua_searcher::Searcher` struct.
    fn add_searcher(&self, modules: HashMap<Cow<'static, str>, Cow<'static, str>>) -> Result<()>;

    /// Like `add_searcher`, but with `modules` values given as paths to files containing
    /// Lua source code to facilitate module reloading.
    fn add_path_searcher<P>(&self, modules: HashMap<Cow<'static, str>, P>) -> Result<()>
    where
        P: 'static + AsRef<Path> + Send;

    /// Like `add_path_searcher`, but with user-provided closure for transforming source
    /// code to Lua.
    fn add_path_searcher_poly<P>(
        &self,
        modules: HashMap<Cow<'static, str>, P>,
        transform: Box<dyn Fn(PathBuf) -> mlua::Result<String> + Send>,
    ) -> Result<()>
    where
        P: 'static + AsRef<Path> + Send;

    /// Like `add_searcher`, but with user-provided closure for `mlua::Lua` setup.
    fn add_closure_searcher(
        &self,
        modules: HashMap<
            Cow<'static, str>,
            Box<dyn Fn(&Lua, Table, &str) -> mlua::Result<Function> + Send>,
        >,
    ) -> Result<()>;

    /// Like `add_searcher`, but with user-provided function for `mlua::Lua` setup.
    fn add_function_searcher(
        &self,
        modules: HashMap<Cow<'static, str>, fn(&Lua, Table, &str) -> mlua::Result<Function>>,
    ) -> Result<()>;

    /// Like `add_searcher`, except `modules` can contain heterogenous strings and paths
    /// indexed by module name.
    fn add_cat_searcher(&self, modules: CatMap) -> Result<()>;
}

impl AddSearcher for Lua {
    fn add_searcher(&self, modules: HashMap<Cow<'static, str>, Cow<'static, str>>) -> Result<()> {
        let globals = self.globals();
        let searchers: Table = globals.get::<Table>("package")?.get("searchers")?;
        let registry_key = self.create_registry_value(globals)?;
        let searcher = Searcher::new(modules, registry_key);
        searchers
            .set(searchers.len()? + 1, searcher)
            .map_err(|e| e.into())
    }

    fn add_path_searcher<P>(&self, modules: HashMap<Cow<'static, str>, P>) -> Result<()>
    where
        P: 'static + AsRef<Path> + Send,
    {
        let transform = Box::new(|path| {
            let mut content = String::new();
            let mut file = File::open(path)
                .map_err(|e| mlua::Error::RuntimeError(format!("io error: {:#?}", e)))?;
            file.read_to_string(&mut content)
                .map_err(|e| mlua::Error::RuntimeError(format!("io error: {:#?}", e)))?;
            Ok(content)
        });
        self.add_path_searcher_poly(modules, transform)
    }

    fn add_path_searcher_poly<P>(
        &self,
        modules: HashMap<Cow<'static, str>, P>,
        transform: Box<dyn Fn(PathBuf) -> mlua::Result<String> + Send>,
    ) -> Result<()>
    where
        P: 'static + AsRef<Path> + Send,
    {
        let globals = self.globals();
        let searchers: Table = globals.get::<Table>("package")?.get("searchers")?;
        let registry_key = self.create_registry_value(globals)?;
        let searcher = PathSearcherPoly::new(modules, registry_key, transform);
        searchers
            .set(searchers.len()? + 1, searcher)
            .map_err(|e| e.into())
    }

    fn add_closure_searcher(
        &self,
        modules: HashMap<
            Cow<'static, str>,
            Box<dyn Fn(&Lua, Table, &str) -> mlua::Result<Function> + Send>,
        >,
    ) -> Result<()> {
        let globals = self.globals();
        let searchers: Table = globals.get::<Table>("package")?.get("searchers")?;
        let registry_key = self.create_registry_value(globals)?;
        let searcher = ClosureSearcher::new(modules, registry_key);
        searchers
            .set(searchers.len()? + 1, searcher)
            .map_err(|e| e.into())
    }

    fn add_function_searcher(
        &self,
        modules: HashMap<Cow<'static, str>, fn(&Lua, Table, &str) -> mlua::Result<Function>>,
    ) -> Result<()> {
        let globals = self.globals();
        let searchers: Table = globals.get::<Table>("package")?.get("searchers")?;
        let registry_key = self.create_registry_value(globals)?;
        let searcher = FunctionSearcher::new(modules, registry_key);
        searchers
            .set(searchers.len()? + 1, searcher)
            .map_err(|e| e.into())
    }

    fn add_cat_searcher(&self, modules: CatMap) -> Result<()> {
        let globals = self.globals();
        let searchers: Table = globals.get::<Table>("package")?.get("searchers")?;
        let registry_key = self.create_registry_value(globals)?;
        let searcher = CatSearcher::new(modules, registry_key);
        searchers
            .set(searchers.len()? + 1, searcher)
            .map_err(|e| e.into())
    }
}
