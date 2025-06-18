use ignore::Walk;
use mlua::{FromLuaMulti, Function, IntoLua, Lua, MultiValue, Table, UserData, Value};
use optional_collections::PushOrInit;
use std::borrow::Cow;
use std::clone::Clone;
use std::convert::{From, TryFrom};
use std::fmt;
use std::fmt::Debug;
use std::iter::Extend;
use std::ops::Index;
use std::string::String;
use std::vec::Vec;

use crate::manifest_error::{ManifestInitError, NamedTextManifestInitError};
use crate::mir::Mir;
use crate::mir_arg::{Dict, MirArg, MirArgs};
use crate::mir_consts::{FROM_LUA_MULTI_EXPECT, INTO_LUA_EXPECT, PARTITIONED_EXPECT};
use crate::mir_error::{
    DictError, DictErrorKind, InputManifestError, MirError, MirErrorKind, MissingError,
    StringErrorKind, UserDataErrorKind,
};
use crate::mir_types::{DictResult, InputManifestResult, InputStringResult, MirResult};
use crate::module::{Module, ModuleFile, ModuleNamedText};
use crate::module_error::{ModuleInitError, ModuleNamedTextInitError};
use crate::module_traits::Name;

/// Position of optional docstring in `Manifest` instantiation input `MultiValue`.
const DOCSTRING_POSITION: usize = 0;

/// `Manifest` can contain either Fennel or Lua text, or file paths presumed to contain
/// Fennel or Lua text.
///
/// N.B. `Module`s in `Manifest` aren't guaranteed to be resolveable to embedded text
/// at comptime unless all `Module`s are of variant `Module::NamedText`.
#[derive(Clone, Debug)]
pub struct Manifest {
    pub docstring: Option<Cow<'static, str>>,
    pub modules: Vec<Module>,
}

impl Manifest {
    pub fn new(docstring: Option<Cow<'static, str>>, modules: Vec<Module>) -> Self {
        Self { docstring, modules }
    }

    pub fn try_from_dir<S>(path: S) -> Result<Manifest, ManifestInitError>
    where
        S: AsRef<str>,
    {
        let modules = Walk::new(path.as_ref())
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                // Get files only.
                e.file_type()
                    .and_then(|file_type| Some(file_type.is_file()))
                    .map_or_else(|| false, |e| e)
            })
            .map(|e| {
                ModuleFile::new(e.into_path(), None)
                    .map_err(|e| ModuleInitError::from(e))
                    .map_err(|e| ManifestInitError::from(e))
            })
            .collect::<Result<Vec<ModuleFile>, ManifestInitError>>()?
            .into_iter()
            .map(|e| Module::File(e))
            .collect();
        Ok(Manifest::new(None, modules))
    }

    pub fn add(&mut self, elem: Module) {
        self.modules.push(elem);
    }

    pub fn append(
        &mut self,
        Manifest {
            docstring: _,
            modules,
        }: &mut Manifest,
    ) {
        self.modules.append(modules);
    }

    pub fn get<'a>(&'a self, name: &str) -> Option<&'a Module> {
        self.modules
            .iter()
            .filter(|module| module.name().eq(name))
            .last()
    }

    pub fn loader(lua: &Lua, env: Table, name: &str) -> mlua::Result<Function> {
        let new = lua.create_function(|lua, multi_value: MultiValue| {
            Ok(Manifest::from_lua_multi(multi_value, lua)?)
        })?;

        let walk = lua.create_function(|_, value: Value| {
            if let Value::String(path) = value {
                let path = &*path.to_str()?;
                Ok(Manifest::try_from_dir(path)?)
            } else {
                let got = mlua_utils::typename(&value);
                Err(mlua::Error::RuntimeError(format!(
                    "Manifest.walk expected string argument but got {}",
                    got
                )))
            }
        })?;

        let tbl = lua.create_table()?;
        tbl.set("new", new)?;
        tbl.set("walk", walk)?;

        let globals = lua.globals();
        globals.set("manifest", tbl)?;

        Ok(lua
            .load("return manifest")
            .set_name(name)
            .set_environment(env)
            .into_function()?)
    }
}

impl Extend<Module> for Manifest {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = Module>,
    {
        for elem in iter {
            self.add(elem);
        }
    }
}

impl Index<usize> for Manifest {
    type Output = Module;

    fn index(&self, index: usize) -> &Self::Output {
        &self.modules[index]
    }
}

impl UserData for Manifest {}

impl FromLuaMulti for Manifest {
    fn from_lua_multi(multi_value: MultiValue, lua: &Lua) -> mlua::Result<Self> {
        let Mir {
            mir_args,
            // XXX: Useful if embedding inspect.lua or similar for `MirError` -> `mlua::Error`.
            registry_keys: _,
        } = Mir::from_lua_multi(multi_value, lua).expect(FROM_LUA_MULTI_EXPECT);

        match Manifest::try_from(mir_args) {
            Ok(manifest) => Ok(manifest),
            // Convert `MirError` into `mlua::Error`.
            Err(e) => {
                let Value::String(s) = e.into_lua(lua).expect(INTO_LUA_EXPECT) else {
                    unreachable!()
                };
                let s = &*s.to_str()?;
                Err(mlua::Error::RuntimeError(s.to_string()))
            }
        }
    }
}

impl TryFrom<MirArgs> for Manifest {
    type Error = MirError;

    fn try_from(mir_args: MirArgs) -> MirResult<Self> {
        // Collect input errors rather than raising an error at first encounter.
        //
        // Improves Lua API ergonomics and error reporting.
        //
        // Stores internal `MirErrorKind` type rather than `mlua::Error` due to the
        // latter's limited expressivity (string only).
        let mut errors: Option<Vec<MirErrorKind>> = None;

        let (good_ip_docstring, good_ip_dicts, bad_dicts, good_ip_manifests): (
            Option<(usize, String)>,
            Option<Vec<(usize, Dict)>>,
            Option<Vec<(usize, Dict)>>,
            Option<Vec<(usize, Manifest)>>,
        ) = process_mir_args(mir_args.0, &mut errors);

        new_manifest(
            good_ip_docstring,
            good_ip_dicts,
            bad_dicts,
            good_ip_manifests,
            errors,
        )
    }
}

fn process_mir_args(
    mir_args: Vec<(usize, MirArg)>,
    errors: &mut Option<Vec<MirErrorKind>>,
) -> (
    // i.e. `good_ip_docstring`
    Option<(usize, String)>,
    // i.e. `good_ip_dicts`
    Option<Vec<(usize, Dict)>>,
    // i.e. `bad_dicts`
    Option<Vec<(usize, Dict)>>,
    // i.e. `good_ip_manifests`
    Option<Vec<(usize, Manifest)>>,
) {
    let (strings, dicts, manifests): (
        Option<Vec<(usize, InputStringResult<String>)>>,
        Option<Vec<(usize, DictResult<Dict>)>>,
        Option<Vec<(usize, InputManifestResult<Manifest>)>>,
    ) = partition_mir_args(mir_args, errors);

    let good_ip_docstring: Option<(usize, String)> = process_strings(strings, errors);

    let data_position = get_data_position(&good_ip_docstring);

    // Keep all `Dict`s around to improve accuracy of error reporting. Erroneous `Dict`s
    // may contain a mix of valid and invalid data.
    let (good_ip_dicts, bad_dicts): (Option<Vec<(usize, Dict)>>, Option<Vec<(usize, Dict)>>) =
        process_dicts(dicts, &data_position, errors);

    let good_ip_manifests: Option<Vec<(usize, Manifest)>> = process_userdatas(manifests, errors);

    (
        good_ip_docstring,
        good_ip_dicts,
        bad_dicts,
        good_ip_manifests,
    )
}

fn partition_mir_args(
    mir_args: Vec<(usize, MirArg)>,
    errors: &mut Option<Vec<MirErrorKind>>,
) -> (
    // i.e. `strings`
    Option<Vec<(usize, InputStringResult<String>)>>,
    // i.e. `dicts`
    Option<Vec<(usize, DictResult<Dict>)>>,
    // i.e. `manifests`
    Option<Vec<(usize, InputManifestResult<Manifest>)>>,
) {
    // Extract inner values from `MirArg` enum variants.
    let mut strings: Option<Vec<(usize, InputStringResult<String>)>> = None;
    let mut dicts: Option<Vec<(usize, DictResult<Dict>)>> = None;
    let mut manifests: Option<Vec<(usize, InputManifestResult<Manifest>)>> = None;

    for (count, mir_arg) in mir_args {
        match mir_arg {
            MirArg::String(string) => {
                strings.push_or_init((count, string));
            }
            MirArg::Dict(dict) => {
                dicts.push_or_init((count, dict));
            }
            MirArg::UserData(manifest) => {
                manifests.push_or_init((count, manifest));
            }
            MirArg::Unsupported(got) => {
                errors.push_or_init(MirErrorKind::Unsupported { count, got });
            }
        }
    }

    (strings, dicts, manifests)
}

fn get_data_position(good_ip_docstring: &Option<(usize, String)>) -> usize {
    // Ascertain whether an optional docstring has been passed.
    let has_docstring = good_ip_docstring.as_ref().is_some();

    // Move data input position after optional docstring, if present.
    if has_docstring {
        DOCSTRING_POSITION + 1
    } else {
        DOCSTRING_POSITION
    }
}

fn process_strings(
    strings: Option<Vec<(usize, InputStringResult<String>)>>,
    errors: &mut Option<Vec<MirErrorKind>>,
) -> Option<(usize, String)> {
    // Categorize string in `DOCSTRING_POSITION` (0) as "in position" (ip); mark all
    // other strings as "out of position" (oop).
    let (ip_docstring, mut oop_strings): (
        Option<(usize, InputStringResult<String>)>,
        Option<Vec<(usize, InputStringResult<String>)>>,
    ) = partition_strings_by_position_validity(strings);

    let mut good_ip_docstring: Option<(usize, String)> = None;

    if let Some((count, docstring)) = ip_docstring {
        match docstring {
            Ok(docstring) => {
                good_ip_docstring = Some((count, docstring));
            }
            Err(e) => {
                // Collect erroneous, in-position strings for error reporting.
                let error = StringErrorKind::Malformed(e);
                let error = MirErrorKind::String { count, error };
                errors.push_or_init(error);
            }
        }
    }

    // Collect out-of-position strings for error reporting.
    if let Some(oop_strings) = oop_strings.take() {
        for (count, string) in oop_strings {
            match string {
                Ok(string) => {
                    let error = StringErrorKind::Unexpected(string);
                    let error = MirErrorKind::String { count, error };
                    errors.push_or_init(error);
                }
                Err(e) => {
                    let error = StringErrorKind::UnexpectedMalformed(e);
                    let error = MirErrorKind::String { count, error };
                    errors.push_or_init(error);
                }
            }
        }
    }

    good_ip_docstring
}

fn partition_strings_by_position_validity(
    mut strings: Option<Vec<(usize, InputStringResult<String>)>>,
) -> (
    // i.e. `ip_docstring`
    Option<(usize, InputStringResult<String>)>,
    // i.e. `oop_strings`
    Option<Vec<(usize, InputStringResult<String>)>>,
) {
    if let Some(strings) = strings.take() {
        let (oop_strings, mut ip_docstrings): (
            Vec<(usize, InputStringResult<String>)>,
            Vec<(usize, InputStringResult<String>)>,
        ) = strings
            .into_iter()
            .partition(|(count, _)| *count > DOCSTRING_POSITION);

        let ip_docstring = if ip_docstrings.len() > 0 {
            let ip_docstring = ip_docstrings.remove(0);

            // Sanity check.
            if !ip_docstrings.is_empty() {
                unreachable!("Unexpectedly got more than one in-position docstring");
            }

            Some(ip_docstring)
        } else {
            None
        };

        let oop_strings = if oop_strings.len() > 0 {
            Some(oop_strings)
        } else {
            None
        };

        (ip_docstring, oop_strings)
    } else {
        (None, None)
    }
}

fn process_dicts(
    dicts: Option<Vec<(usize, DictResult<Dict>)>>,
    data_position: &usize,
    errors: &mut Option<Vec<MirErrorKind>>,
) -> (
    // i.e. `good_ip_dicts`
    Option<Vec<(usize, Dict)>>,
    // i.e. `bad_dicts`
    Option<Vec<(usize, Dict)>>,
) {
    let (ip_dicts, oop_dicts): (
        Option<Vec<(usize, DictResult<Dict>)>>,
        Option<Vec<(usize, DictResult<Dict>)>>,
    ) = partition_dicts_by_position_validity(dicts, data_position);

    // Collect in-position dicts with erroneous keypairs for error reporting.
    let (good_ip_dicts, mut bad_ip_dicts): (
        Option<Vec<(usize, Dict)>>,
        Option<Vec<(usize, DictError)>>,
    ) = partition_dicts_by_keypair_validity(ip_dicts);

    let mut bad_dicts: Option<Vec<(usize, Dict)>> = None;

    if let Some(bad_ip_dicts) = bad_ip_dicts.take() {
        for (count, error) in bad_ip_dicts {
            let DictError::Malformed { dict, unsupported } = error;
            let error = DictErrorKind::Malformed(unsupported);
            let error = MirErrorKind::Dict { count, error };
            errors.push_or_init(error);
            bad_dicts.push_or_init((count, dict));
        }
    }

    let (mut good_oop_dicts, mut bad_oop_dicts): (
        Option<Vec<(usize, Dict)>>,
        Option<Vec<(usize, DictError)>>,
    ) = partition_dicts_by_keypair_validity(oop_dicts);

    if let Some(bad_oop_dicts) = bad_oop_dicts.take() {
        // Collect out-of-position dicts with erroneous keypairs for error reporting.
        for (count, error) in bad_oop_dicts {
            let DictError::Malformed { dict, unsupported } = error;
            let error = DictErrorKind::UnexpectedMalformed(unsupported);
            let error = MirErrorKind::Dict { count, error };
            errors.push_or_init(error);
            bad_dicts.push_or_init((count, dict));
        }
    }

    if let Some(good_oop_dicts) = good_oop_dicts.take() {
        // Collect out-of-position dicts for error reporting.
        for (count, dict) in good_oop_dicts {
            let error = DictErrorKind::Unexpected;
            let error = MirErrorKind::Dict { count, error };
            errors.push_or_init(error);
            bad_dicts.push_or_init((count, dict));
        }
    }

    (good_ip_dicts, bad_dicts)
}

fn partition_dicts_by_position_validity(
    mut dicts: Option<Vec<(usize, DictResult<Dict>)>>,
    data_position: &usize,
) -> (
    // i.e. `ip_dicts`
    Option<Vec<(usize, DictResult<Dict>)>>,
    // i.e. `oop_dicts`
    Option<Vec<(usize, DictResult<Dict>)>>,
) {
    if let Some(dicts) = dicts.take() {
        let (oop_dicts, ip_dicts): (
            Vec<(usize, DictResult<Dict>)>,
            Vec<(usize, DictResult<Dict>)>,
        ) = dicts
            .into_iter()
            .partition(|(count, _)| count < data_position);

        let ip_dicts = if ip_dicts.len() > 0 {
            Some(ip_dicts)
        } else {
            None
        };

        let oop_dicts = if oop_dicts.len() > 0 {
            Some(oop_dicts)
        } else {
            None
        };

        (ip_dicts, oop_dicts)
    } else {
        (None, None)
    }
}

fn partition_dicts_by_keypair_validity(
    mut dicts: Option<Vec<(usize, DictResult<Dict>)>>,
) -> (
    // i.e. `good_ip_dicts`
    Option<Vec<(usize, Dict)>>,
    // i.e. `bad_ip_dicts`
    Option<Vec<(usize, DictError)>>,
) {
    if let Some(dicts) = dicts.take() {
        let (good_dicts, bad_dicts): (
            Vec<(usize, DictResult<Dict>)>,
            Vec<(usize, DictResult<Dict>)>,
        ) = dicts
            .into_iter()
            .partition(|(_, v)| if let Ok(_) = v { true } else { false });

        let good_dicts = if good_dicts.len() > 0 {
            // Unwrap `Ok` from `good_dicts` elements.
            let good_dicts = good_dicts
                .into_iter()
                .map(|(count, good_dict)| (count, good_dict.expect(PARTITIONED_EXPECT)))
                .collect();

            Some(good_dicts)
        } else {
            None
        };

        let bad_dicts = if bad_dicts.len() > 0 {
            // Unwrap `Err` from `bad_dicts` elements.
            let bad_dicts = bad_dicts
                .into_iter()
                .map(|(count, bad_dict)| (count, bad_dict.expect_err(PARTITIONED_EXPECT)))
                .collect();

            Some(bad_dicts)
        } else {
            None
        };

        (good_dicts, bad_dicts)
    } else {
        (None, None)
    }
}

fn process_userdatas(
    mut manifests: Option<Vec<(usize, InputManifestResult<Manifest>)>>,
    errors: &mut Option<Vec<MirErrorKind>>,
) -> Option<Vec<(usize, Manifest)>> {
    if let Some(manifests) = manifests.take() {
        let (good_manifests, bad_manifests): (
            Vec<(usize, InputManifestResult<Manifest>)>,
            Vec<(usize, InputManifestResult<Manifest>)>,
        ) = manifests
            .into_iter()
            .partition(|(_, v)| if let Ok(_) = v { true } else { false });

        let good_manifests = if good_manifests.len() > 0 {
            // Unwrap `Ok` from `good_manifests` elements.
            let good_manifests = good_manifests
                .into_iter()
                .map(|(count, good_manifest)| (count, good_manifest.expect(PARTITIONED_EXPECT)))
                .collect();

            Some(good_manifests)
        } else {
            None
        };

        if bad_manifests.len() > 0 {
            // Unwrap `Err` from `bad_manifests` elements.
            let bad_manifests: Vec<(usize, InputManifestError)> = bad_manifests
                .into_iter()
                .map(|(count, bad_manifest)| (count, bad_manifest.expect_err(PARTITIONED_EXPECT)))
                .collect();

            for (count, _) in bad_manifests {
                let error = UserDataErrorKind::Unexpected;
                let error = MirErrorKind::UserData { count, error };
                errors.push_or_init(error);
            }
        }

        good_manifests
    } else {
        None
    }
}

fn new_manifest(
    good_ip_docstring: Option<(usize, String)>,
    good_ip_dicts: Option<Vec<(usize, Dict)>>,
    _: Option<Vec<(usize, Dict)>>,
    good_ip_manifests: Option<Vec<(usize, Manifest)>>,
    errors: Option<Vec<MirErrorKind>>,
) -> MirResult<Manifest> {
    if let Some(errors) = errors {
        return Err(MirError::Input { errors });
    }

    let mut manifest = Manifest {
        docstring: None,
        modules: Vec::new(),
    };

    match (good_ip_dicts, good_ip_manifests) {
        (None, None) => {
            let mut errors: Vec<MirErrorKind> = Vec::new();
            errors.push(MirErrorKind::from(MissingError::TablePathOrUserData));
            return Err(MirError::Input { errors });
        }
        (Some(good_ip_dicts), Some(good_ip_manifests)) => {
            for (_, good_ip_dict) in good_ip_dicts {
                let module =
                    Module::try_from(good_ip_dict).map_err(|e| MirError::ModuleInitError(e))?;
                manifest.modules.push(module);
            }
            // append modules from Manifest
            for (_, mut good_ip_manifest) in good_ip_manifests {
                manifest.modules.append(&mut good_ip_manifest.modules);
            }
        }
        (Some(good_ip_dicts), None) => {
            for (_, good_ip_dict) in good_ip_dicts {
                let module =
                    Module::try_from(good_ip_dict).map_err(|e| MirError::ModuleInitError(e))?;
                manifest.modules.push(module);
            }
        }
        (None, Some(good_ip_manifests)) => {
            for (_, mut good_ip_manifest) in good_ip_manifests {
                manifest.modules.append(&mut good_ip_manifest.modules);
            }
        }
    }

    if let Some((_, docstring)) = good_ip_docstring {
        manifest.docstring = Some(docstring.into());
    }

    Ok(manifest)
}

impl fmt::Display for Manifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = fmt::<Module>("Manifest", &self.docstring, &self.modules);
        write!(f, "{}", res)
    }
}

#[derive(Clone, Debug)]
pub struct NamedTextManifest {
    pub docstring: Option<Cow<'static, str>>,
    pub modules: Vec<ModuleNamedText>,
}

impl NamedTextManifest {
    pub fn new(docstring: Option<Cow<'static, str>>, modules: Vec<ModuleNamedText>) -> Self {
        Self { docstring, modules }
    }

    pub fn try_from_dir<S>(path: S) -> Result<NamedTextManifest, NamedTextManifestInitError>
    where
        S: AsRef<str>,
    {
        NamedTextManifest::try_from(Manifest::try_from_dir(path)?)
    }

    pub fn add(&mut self, elem: ModuleNamedText) {
        self.modules.push(elem);
    }

    pub fn append(
        &mut self,
        NamedTextManifest {
            docstring: _,
            modules,
        }: &mut NamedTextManifest,
    ) {
        self.modules.append(modules);
    }

    pub fn get<'a>(&'a self, name: &str) -> Option<&'a ModuleNamedText> {
        self.modules
            .iter()
            .filter(|module| module.name().eq(name))
            .last()
    }
}

impl Extend<ModuleNamedText> for NamedTextManifest {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = ModuleNamedText>,
    {
        for elem in iter {
            self.add(elem);
        }
    }
}

impl Index<usize> for NamedTextManifest {
    type Output = ModuleNamedText;

    fn index(&self, index: usize) -> &Self::Output {
        &self.modules[index]
    }
}

impl TryFrom<Manifest> for NamedTextManifest {
    type Error = NamedTextManifestInitError;

    fn try_from(
        Manifest { docstring, modules }: Manifest,
    ) -> Result<Self, NamedTextManifestInitError> {
        let modules = modules
            .into_iter()
            .map(|module| match module {
                Module::File(m) => ModuleNamedText::try_from(m),
                Module::NamedFile(m) => ModuleNamedText::try_from(m),
                Module::NamedText(m) => Ok(m),
            })
            .collect::<Result<Vec<ModuleNamedText>, ModuleNamedTextInitError>>()?;
        Ok(Self { docstring, modules })
    }
}

impl fmt::Display for NamedTextManifest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let res = fmt::<ModuleNamedText>("NamedTextManifest", &self.docstring, &self.modules);
        write!(f, "{}", res)
    }
}

fn fmt<T>(type_name: &str, docstring: &Option<Cow<'static, str>>, modules: &Vec<T>) -> String
where
    T: fmt::Display,
{
    let mut v = String::new();
    for module in modules {
        String::push_str(&mut v, &format!("{}, ", module));
    }

    // Trim trailing whitespace.
    _ = v.pop();
    // Trim trailing comma separator.
    _ = v.pop();

    if let Some(docstring) = docstring.as_ref() {
        format!(
            "{} {{ docstring: {:?}, modules: vec![{}] }}",
            type_name, docstring, v
        )
    } else {
        format!("{} {{ modules: vec![{}] }}", type_name, v)
    }
}
