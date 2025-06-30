mod error;
mod manifest;

pub mod prelude {
    pub use crate::error::CompiledNamedTextManifestInitError;
    pub use crate::manifest::CompiledNamedTextManifest;
}

pub use crate::error::CompiledNamedTextManifestInitError;
pub use crate::manifest::CompiledNamedTextManifest;
