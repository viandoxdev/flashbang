use std::io::Read;

#[cfg(feature = "github")]
use crate::error::CoreError;

pub trait CacheProvider: Send + Sync + 'static {
    #[cfg(feature = "github")]
    fn get_sha(&self) -> Result<String, CoreError>;
    #[cfg(feature = "github")]
    fn save_sha(&self, sha: String) -> Result<(), CoreError>;

    fn get_tarball(&self) -> Result<Box<dyn Read>, CoreError>;
    fn save_tarball(&self, data: &mut dyn Read) -> Result<(), CoreError>;
}
