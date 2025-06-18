use std::borrow::Cow;
use std::result;

use crate::error::Error;

pub(crate) type CatMap = io_cat::CatMap<Cow<'static, str>>;
pub type Result<A> = result::Result<A, Error>;
