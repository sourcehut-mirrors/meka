use derive_builder::Builder;
use mlua::{AnyUserData, Table, Value};
use mlua_utils::{IntoCharArray, TryIntoString};
use optional_collections::PushOrInit;
use std::convert::{From, TryFrom};
use std::vec::Vec;

use crate::manifest::Manifest;
use crate::mir_consts::PAIRS_EXPECT;
use crate::mir_error::{
    DictError, DictKeyPairError, DictNameError, DictPathError, DictTextError, DictTypeError,
    InputManifestError, InputStringError,
};
use crate::mir_types::{
    DictNameResult, DictPathResult, DictResult, DictTextResult, DictTypeResult,
    InputManifestResult, InputStringResult,
};

/// A collection of `Result`-wrapped arguments paired with a numeric "count".
///
/// "Count" is the `usize` order in which the argument appeared within the `mlua::MultiValue`
/// from which it was instantiated.
pub struct MirArgs(pub Vec<(usize, MirArg)>);

/// MIR Argument.
///
/// Categorizes the acceptable `Manifest` instantiation input formats passed in via an
/// `mlua::MultiValue`.
///
/// The values wrapped in `Result` are wrapped as such to preserve context, e.g. that we
/// received an `mlua::String` or `mlua::Table` which proved to contain either valid or
/// invalid input during processing. Said context is useful in enforcing the order of
/// arguments during `Manifest` instantiation.
#[derive(Debug)]
pub enum MirArg {
    /// `mlua::String`.
    ///
    /// Wrapped in an `InputStringResult` to represent erroneous input, e.g. invalid
    /// UTF-8.
    String(InputStringResult<String>),

    /// `mlua::Table`.
    ///
    /// Wrapped in a `DictResult` to represent erroneous input, e.g. unsupported keys.
    Dict(DictResult<Dict>),

    /// `mlua::UserData`.
    ///
    /// Wrapped in an `InputManifestResult` to represent erroneous input, e.g. userdata
    /// which doesn't correspond to `Manifest`.
    UserData(InputManifestResult<Manifest>),

    /// An unsupported value.
    ///
    /// The inner value is the unsupported value's Lua type.
    Unsupported(&'static str),
}

impl From<Value> for MirArg {
    fn from(value: Value) -> Self {
        match value {
            // Got `mlua::Table`. Interpreting as `Dict`.
            Value::Table(table) => {
                // Walk `table` pairs, using builder pattern to extract data.
                let dict = Dict::try_from(table);
                MirArg::Dict(dict)
            }

            // Got `mlua::UserData`
            Value::UserData(ud) => {
                let manifest = Manifest::try_from(ud);
                MirArg::UserData(manifest)
            }

            // Got `Value` variant other than `Value::Table` or `Value::UserData`. Assuming
            // `Value::String`, because no other input type is supported.
            value => match value.try_into_string() {
                Ok(content) => MirArg::String(Ok(content)),
                Err(e) => match e {
                    // We were passed an `mlua::String`, but its contents couldn't be
                    // converted to UTF-8.
                    mlua_utils::InputStringError::MalformedString { content } => {
                        let error = Err(InputStringError::MalformedString { content });
                        MirArg::String(error)
                    }

                    // If we got here, we weren't passed an `mlua::String`, an `mlua::Table`,
                    // or an `mlua::UserData`. Hence, we must've been passed an unsupported
                    // input type.
                    mlua_utils::InputStringError::MissingString { got } => MirArg::Unsupported(got),
                },
            },
        }
    }
}

/// `mlua::Table` with exclusively non-numeric keys.
///
/// May contain string keys whose associated values are useful in `Manifest` instantiation.
#[derive(Clone, Debug, Builder)]
pub struct Dict {
    /// Did the input table include a valid `name` keypair?
    #[builder(setter(strip_option), default)]
    pub name: Option<String>,

    /// Did the input table include a valid `path` keypair?
    #[builder(setter(strip_option), default)]
    pub path: Option<String>,

    /// Did the input table include a valid `text` keypair?
    #[builder(setter(strip_option), default)]
    pub text: Option<String>,

    /// Did the input table include a valid `type` keypair?
    #[builder(setter(strip_option), default)]
    pub file_type: Option<String>,
}

impl Dict {
    fn validate(&self) -> Result<(), DictKeyPairError> {
        if let None = &self.path {
            if let None = &self.text {
                return Err(DictKeyPairError::MissingRequiredKey);
            }
        }
        if let Some(_) = &self.text {
            if let None = &self.name {
                return Err(DictKeyPairError::MissingRequiredNameKey);
            }
            if let None = &self.file_type {
                return Err(DictKeyPairError::MissingRequiredTypeKey);
            }
            if let Some(_) = &self.path {
                return Err(DictKeyPairError::MutuallyExclusiveKeys);
            }
        }
        Ok(())
    }
}

impl TryFrom<Table> for Dict {
    type Error = DictError;

    fn try_from(table: Table) -> DictResult<Self> {
        let mut builder = DictBuilder::default();

        // Collection of unsupported keypairs found, represented as `DictKeyPairError`s.
        let mut unsupported: Option<Vec<DictKeyPairError>> = None;

        for pairs in table.pairs::<Value, Value>() {
            handle_table_pairs(pairs, &mut builder, &mut unsupported);
        }

        let dict = builder.build().expect("DictBuilder unexpectedly failed");

        // `name` and `type` fields are optional, but one of either `path` or `text` must be
        // given; `path` and `text` fields are mutually exclusive.
        match dict.validate() {
            Ok(_) => {
                if let Some(unsupported) = unsupported {
                    // This `Dict` contains unsupported keypairs, and is hence erroneous.
                    Err(DictError::Malformed { dict, unsupported })
                } else {
                    Ok(dict)
                }
            }
            Err(e) => {
                unsupported.push_or_init(e);
                let unsupported = unsupported.unwrap();
                Err(DictError::Malformed { dict, unsupported })
            }
        }
    }
}

struct Name(String);

impl TryFrom<Value> for Name {
    type Error = DictNameError;

    fn try_from(value: Value) -> DictNameResult<Self> {
        match value.try_into_string() {
            Ok(name) => Ok(Name(name)),
            Err(e) => match e {
                mlua_utils::InputStringError::MalformedString { content } => {
                    Err(DictNameError::MalformedString { name: content })
                }
                mlua_utils::InputStringError::MissingString { got } => {
                    Err(DictNameError::MissingString { got })
                }
            },
        }
    }
}

impl From<Name> for String {
    fn from(name: Name) -> Self {
        // Unwrap `String` from `Name`.
        name.0
    }
}

struct PathStr(String);

impl TryFrom<Value> for PathStr {
    type Error = DictPathError;

    fn try_from(value: Value) -> DictPathResult<Self> {
        match value.try_into_string() {
            Ok(path) => Ok(PathStr(path)),
            Err(e) => match e {
                mlua_utils::InputStringError::MalformedString { content } => {
                    Err(DictPathError::MalformedString { path: content })
                }
                mlua_utils::InputStringError::MissingString { got } => {
                    Err(DictPathError::MissingString { got })
                }
            },
        }
    }
}

impl From<PathStr> for String {
    fn from(path: PathStr) -> Self {
        // Unwrap `String` from `PathStr`.
        path.0
    }
}

struct Text(String);

impl TryFrom<Value> for Text {
    type Error = DictTextError;

    fn try_from(value: Value) -> DictTextResult<Self> {
        match value.try_into_string() {
            Ok(text) => Ok(Text(text)),
            Err(e) => match e {
                mlua_utils::InputStringError::MalformedString { content } => {
                    Err(DictTextError::MalformedString { text: content })
                }
                mlua_utils::InputStringError::MissingString { got } => {
                    Err(DictTextError::MissingString { got })
                }
            },
        }
    }
}

impl From<Text> for String {
    fn from(text: Text) -> Self {
        // Unwrap `String` from `Text`.
        text.0
    }
}

struct Type(String);

impl TryFrom<Value> for Type {
    type Error = DictTypeError;

    fn try_from(value: Value) -> DictTypeResult<Self> {
        match value.try_into_string() {
            Ok(file_type) => Ok(Type(file_type)),
            Err(e) => match e {
                mlua_utils::InputStringError::MalformedString { content } => {
                    Err(DictTypeError::MalformedString { file_type: content })
                }
                mlua_utils::InputStringError::MissingString { got } => {
                    Err(DictTypeError::MissingString { got })
                }
            },
        }
    }
}

impl From<Type> for String {
    fn from(file_type: Type) -> Self {
        // Unwrap `String` from `Type`.
        file_type.0
    }
}

fn handle_table_pairs(
    pairs: mlua::Result<(Value, Value)>,
    builder: &mut DictBuilder,
    unsupported: &mut Option<Vec<DictKeyPairError>>,
) {
    match pairs.expect(PAIRS_EXPECT) {
        // Found `mlua::String` key.
        (Value::String(key), value) => {
            handle_string_key(key, value, builder, unsupported);
        }

        // Found unsupported key.
        (key, _) => {
            let got = mlua_utils::typename(&key);
            let error = DictKeyPairError::MissingKeyString { got };
            unsupported.push_or_init(error);
        }
    }
}

fn handle_string_key(
    key: mlua::String,
    value: Value,
    builder: &mut DictBuilder,
    unsupported: &mut Option<Vec<DictKeyPairError>>,
) {
    match key.to_str() {
        Ok(key) => match &*key {
            "name" => handle_name_value(value, builder, unsupported),
            "path" => handle_path_value(value, builder, unsupported),
            "text" => handle_text_value(value, builder, unsupported),
            "type" => handle_type_value(value, builder, unsupported),
            key => handle_unexpected_value(key, value, unsupported),
        },
        _ => {
            let error = DictKeyPairError::MalformedKeyString {
                key: key.into_char_array(),
            };
            unsupported.push_or_init(error);
        }
    }
}

fn handle_name_value(
    value: Value,
    builder: &mut DictBuilder,
    unsupported: &mut Option<Vec<DictKeyPairError>>,
) {
    match Name::try_from(value) {
        Ok(name) => {
            // Convert `Name` into `String`.
            let name = String::from(name);
            builder.name(name);
        }
        Err(e) => {
            // Convert `DictNameError` into `DictKeyPairError`.
            let error = DictKeyPairError::from(e);
            unsupported.push_or_init(error);
        }
    }
}

fn handle_path_value(
    value: Value,
    builder: &mut DictBuilder,
    unsupported: &mut Option<Vec<DictKeyPairError>>,
) {
    match PathStr::try_from(value) {
        Ok(path) => {
            // Convert `PathStr` into `String`.
            let path = String::from(path);
            builder.path(path);
        }
        Err(e) => {
            // Convert `DictPathError` into `DictKeyPairError`.
            let error = DictKeyPairError::from(e);
            unsupported.push_or_init(error);
        }
    }
}

fn handle_text_value(
    value: Value,
    builder: &mut DictBuilder,
    unsupported: &mut Option<Vec<DictKeyPairError>>,
) {
    match Text::try_from(value) {
        Ok(text) => {
            // Convert `Text` into `String`.
            let text = String::from(text);
            builder.text(text);
        }
        Err(e) => {
            // Convert `DictTextError` into `DictKeyPairError`.
            let error = DictKeyPairError::from(e);
            unsupported.push_or_init(error);
        }
    }
}

fn handle_type_value(
    value: Value,
    builder: &mut DictBuilder,
    unsupported: &mut Option<Vec<DictKeyPairError>>,
) {
    match Type::try_from(value) {
        Ok(file_type) => {
            // Convert `Type` into `String`.
            let file_type = String::from(file_type);
            builder.file_type(file_type);
        }
        Err(e) => {
            // Convert `DictTypeError` into `DictKeyPairError`.
            let error = DictKeyPairError::from(e);
            unsupported.push_or_init(error);
        }
    }
}

fn handle_unexpected_value(
    key: &str,
    value: Value,
    unsupported: &mut Option<Vec<DictKeyPairError>>,
) {
    let key = key.to_string();
    let value = mlua_utils::typename(&value);
    let error = DictKeyPairError::UnexpectedKeyString { key, value };
    unsupported.push_or_init(error);
}

impl TryFrom<AnyUserData> for Manifest {
    type Error = InputManifestError;

    fn try_from(ud: AnyUserData) -> InputManifestResult<Self> {
        match ud.borrow::<Manifest>() {
            Ok(manifest) => {
                let manifest = (*manifest).clone();
                Ok(manifest)
            }
            _ => Err(InputManifestError::MalformedManifestUserData),
        }
    }
}
