use std::result::Result;

use crate::mir_error::{
    DictError, DictNameError, DictPathError, DictTextError, DictTypeError, InputManifestError,
    InputStringError, MirError,
};

pub type DictResult<A> = Result<A, DictError>;
pub type DictNameResult<A> = Result<A, DictNameError>;
pub type DictPathResult<A> = Result<A, DictPathError>;
pub type DictTextResult<A> = Result<A, DictTextError>;
pub type DictTypeResult<A> = Result<A, DictTypeError>;
pub type MirResult<A> = Result<A, MirError>;
pub type InputManifestResult<A> = Result<A, InputManifestError>;
pub type InputStringResult<A> = Result<A, InputStringError>;
