mod compile;
mod error;
mod types;

pub mod prelude {
    pub use crate::compile::Compile;
    pub use crate::error::Error;
    pub use crate::types::Result;
}

pub use crate::compile::Compile;
pub use crate::error::Error;
pub use crate::types::Result;
