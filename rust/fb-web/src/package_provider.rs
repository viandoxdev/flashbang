use std::{
    io::{Cursor, Read},
    path::{Path, PathBuf}, sync::Arc,
};

use fb_core::{
    error::{AsCoreError, CoreError},
    packages::PackageProvider,
    typst::{
        diag::{FileError, FileResult},
        ecow::EcoString,
        foundations::Bytes,
        syntax::{FileId, Source, package::PackageSpec},
    },
    world::WorldState,
};
use parking_lot::Mutex;
use zip::{ZipArchive, read::ZipFile, result::ZipError};

pub struct ZippedPackageProvider {
    data: Mutex<ZipArchive<Cursor<Arc<[u8]>>>>,
    packages_root: PathBuf,
}

impl ZippedPackageProvider {
    pub fn new(data: Arc<[u8]>) -> Result<Self, CoreError> {
        Ok(Self {
            data: Mutex::new(ZipArchive::new(Cursor::new(data)).context(Some("Zip"))?),
            packages_root: PathBuf::from("packages"),
        })
    }

    fn package_directory(&self, spec: &PackageSpec) -> PathBuf {
        self.packages_root
            .join(format!("{}/{}/{}", spec.namespace, spec.name, spec.version))
    }

    fn with_file<T>(
        &self,
        id: FileId,
        fun: impl FnOnce(ZipFile<'_, Cursor<Arc<[u8]>>>, &Path) -> FileResult<T>,
    ) -> FileResult<T> {
        let spec = id.package().expect("File doesn't belong to any package");
        let path = self
            .package_directory(spec)
            .join(id.vpath().as_rootless_path());
        let mut data = self.data.lock();
        let file = data.by_path(&path).file_context(Some(&path))?;

        fun(file, &path)
    }
}

trait ZipErrorToFileError<T> {
    fn file_context(self, path: Option<&Path>) -> Result<T, FileError>;
}

impl<T> ZipErrorToFileError<T> for Result<T, ZipError> {
    fn file_context(self, path: Option<&Path>) -> Result<T, FileError> {
        match self {
            Ok(v) => Ok(v),
            Err(err) => Err({
                let path = path.unwrap_or(Path::new(""));
                match err {
                    ZipError::Io(io) => FileError::from_io(io, path),
                    ZipError::FileNotFound => FileError::NotFound(path.to_path_buf()),
                    _ => FileError::Other(Some(EcoString::from(err.to_string()))),
                }
            }),
        }
    }
}

impl PackageProvider for ZippedPackageProvider {
    fn get_package_source(&self, id: FileId, _world: &WorldState) -> FileResult<Source> {
        self.with_file(id, |mut file, path| {
            let mut buf = String::new();
            file.read_to_string(&mut buf)
                .map_err(|err| FileError::from_io(err, path))?;

            Ok(Source::new(id, buf))
        })
    }

    fn get_package_file(&self, id: FileId, _world: &WorldState) -> FileResult<Bytes> {
        self.with_file(id, |mut file, path| {
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)
                .map_err(|err| FileError::from_io(err, path))?;

            Ok(Bytes::new(buf))
        })
    }
}
