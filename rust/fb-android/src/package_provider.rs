use std::{path::PathBuf, sync::Arc};

use fb_core::{
    packages::PackageProvider,
    typst::{
        diag::{FileError, FileResult},
        foundations::Bytes,
        syntax::{FileId, Source, package::PackageSpec},
    },
    world::{FileSlot, WorldState},
};
use parking_lot::Mutex;
use reqwest::StatusCode;

/// The default Typst registry.
pub const DEFAULT_REGISTRY: &str = "https://packages.typst.org";

/// The public namespace in the default Typst registry.
pub const DEFAULT_NAMESPACE: &str = "preview";

pub struct DownloadingPackageProvider {
    packages_path: PathBuf,
    client: reqwest::blocking::Client,
    cache_groups: Arc<Mutex<Vec<String>>>,
}

impl DownloadingPackageProvider {
    pub fn new(packages_path: PathBuf, cache_groups: Arc<Mutex<Vec<String>>>) -> Self {
        Self {
            packages_path,
            client: reqwest::blocking::Client::new(),
            cache_groups,
        }
    }
}

impl PackageProvider for DownloadingPackageProvider {
    fn get_package_source(&self, id: FileId, world: &WorldState) -> FileResult<Source> {
        self.download_file(id, world, |path| {
            let content =
                std::fs::read_to_string(&path).map_err(|err| FileError::from_io(err, &path))?;
            let source = Source::new(id, content);

            Ok(FileSlot::with_source(id, source))
        })?
        .source()
    }

    fn get_package_file(&self, id: FileId, world: &WorldState) -> FileResult<Bytes> {
        self.download_file(id, world, |path| {
            let bytes =
                Bytes::new(std::fs::read(&path).map_err(|err| FileError::from_io(err, &path))?);
            Ok(FileSlot::with_bytes(id, bytes))
        })?
        .bytes()
    }
}

impl DownloadingPackageProvider {
    /// Loads a package's file into the worldState if not already loaded, downloading the package
    /// if needed.
    fn download_file(
        &self,
        id: FileId,
        world: &WorldState,
        slot: impl FnOnce(PathBuf) -> FileResult<FileSlot>,
    ) -> FileResult<FileSlot> {
        if let Some(slot) = world.get_file(&id) {
            return Ok(slot);
        }

        let spec = id
            .package()
            .expect("PackageProvider can't provide files that aren't from packages");

        let package_path = self.get_package_directory(spec);

        // Make sure the package has been downloaded
        self.download_package(spec).map_err(|err| FileError::from_io(err, &package_path))?;

        let path = package_path
            .join(id.vpath().as_rootless_path());

        let slot = slot(path)?;

        world.load_file(slot.clone());

        Ok(slot)
    }

    fn get_package_directory(&self, spec: &PackageSpec) -> PathBuf {
        self.packages_path
            .join(format!("{}/{}/{}", spec.namespace, spec.name, spec.version))
    }

    fn package_is_downloaded(&self, spec: &PackageSpec) -> bool {
        let path  = self.get_package_directory(spec);

        std::fs::exists(path).unwrap_or_default()
    }

    /// Download package to cache
    fn download_package(&self, spec: &PackageSpec) -> Result<(), std::io::Error> {
        if self.package_is_downloaded(spec) {
            return Ok(());
        }

        if spec.namespace != DEFAULT_NAMESPACE {
            return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
        }

        let url = format!(
            "{DEFAULT_REGISTRY}/{DEFAULT_NAMESPACE}/{}-{}.tar.gz",
            spec.name, spec.version
        );
        let dir = self.get_package_directory(spec);

        let data = match self.client.get(&url).send() {
            Ok(data) => match data.bytes() {
                Ok(data) => data,
                Err(err) => return Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
            },
            Err(err) if err.status() == Some(StatusCode::NOT_FOUND) => {
                return Err(std::io::Error::new(std::io::ErrorKind::NotFound, err));
            }
            Err(err) => return Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
        };

        std::fs::create_dir_all(&dir)?;

        let decompressed = flate2::read::GzDecoder::new(&data[..]);
        tar::Archive::new(decompressed)
            .unpack(&dir)
            .map_err(|err| {
                std::fs::remove_dir_all(&dir).ok();
                std::io::Error::new(std::io::ErrorKind::Other, err)
            })?;

        self.cache_groups.lock().push(dir.to_string_lossy().to_string());

        Ok(())
    }
}
