mod error;
mod searcher;
mod types;

pub mod prelude {
    pub use crate::error::Error;
    pub use crate::searcher::AddSearcher;
    pub use crate::types::Result;
}

pub use crate::error::Error;
pub use crate::searcher::AddSearcher;
pub use crate::types::Result;
