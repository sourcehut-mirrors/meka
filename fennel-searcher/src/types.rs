use std::result;

use crate::error::Error;

pub type Result<A> = result::Result<A, Error>;
