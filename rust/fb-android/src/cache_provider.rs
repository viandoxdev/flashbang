use std::path::PathBuf;

use fb_core::cache::CacheProvider;

const SHA_FILE: &str = "sha";
const TARBALL_FILE: &str = "tarball.tar.gz";

pub struct FileSystemCacheProvider {
    cache_path: PathBuf,
    sha_path: PathBuf,
    tarball_path: PathBuf,
}

impl FileSystemCacheProvider {
    pub fn new(cache_path: PathBuf) -> Self {
        Self {
            sha_path: cache_path.join(SHA_FILE),
            tarball_path: cache_path.join(TARBALL_FILE),
            cache_path,
        }
    }
}

impl CacheProvider for FileSystemCacheProvider {
    fn get_sha(&self) -> Result<String, fb_core::error::CoreError> {
        Ok(std::fs::read_to_string(&self.sha_path)?)
    }
    fn save_sha(&self, sha: String) -> Result<(), fb_core::error::CoreError> {
        std::fs::create_dir_all(&self.cache_path)?;
        std::fs::write(&self.sha_path, sha)?;

        Ok(())
    }
    fn get_tarball(&self) -> Result<Box<dyn std::io::Read>, fb_core::error::CoreError> {
        Ok(Box::new(std::fs::File::open(&self.tarball_path)?))
    }
    fn save_tarball(&self, data: &mut dyn std::io::Read) -> Result<(), fb_core::error::CoreError> {
        std::fs::create_dir_all(&self.cache_path)?;
        let mut dest = std::fs::File::create(&self.tarball_path)?;

        std::io::copy(data, &mut dest)?;

        Ok(())
    }
}
