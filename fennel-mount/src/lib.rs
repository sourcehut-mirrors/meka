mod error;
mod mount;
mod types;

pub mod prelude {
    pub use crate::error::Error;
    pub use crate::mount::Mount;
    pub use crate::types::Result;
}

pub use crate::error::Error;
pub use crate::mount::Mount;
pub use crate::types::Result;
