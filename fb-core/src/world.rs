use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    fs,
    io::Cursor,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use itertools::Itertools;
use parking_lot::Mutex;
use reqwest::{StatusCode, blocking::Client};
use typst::{
    Library, LibraryExt, World as TypstWorld,
    diag::{FileError, FileResult},
    foundations::Bytes,
    layout::{Page, PagedDocument},
    syntax::{FileId, Source, VirtualPath, package::PackageSpec},
    text::{Font, FontBook},
    utils::LazyHash,
};
use typst_kit::fonts::{FontSearcher, FontSlot as TypstFontSlot};
use walkdir::WalkDir;

use crate::{
    Core,
    cards::{CardInfo, CardSource, SourceConfig},
    error::CoreError,
    github::GithubAPI,
};

/// The default Typst registry.
pub const DEFAULT_REGISTRY: &str = "https://packages.typst.org";

/// The public namespace in the default Typst registry.
pub const DEFAULT_NAMESPACE: &str = "preview";

enum FontSlot {
    Typst(TypstFontSlot),
    Extra(Font),
}

impl FontSlot {
    fn get(&self) -> Option<Font> {
        match self {
            Self::Typst(slot) => slot.get(),
            Self::Extra(font) => Some(font.clone()),
        }
    }
}

impl From<TypstFontSlot> for FontSlot {
    fn from(value: TypstFontSlot) -> Self {
        Self::Typst(value)
    }
}

#[derive(Debug, Clone)]
struct FileSlot {
    id: FileId,
    source: Arc<OnceLock<FileResult<Source>>>,
    bytes: Arc<OnceLock<FileResult<Bytes>>>,
}

impl FileSlot {
    fn new(id: FileId) -> Self {
        Self {
            id,
            source: Arc::new(OnceLock::new()),
            bytes: Arc::new(OnceLock::new()),
        }
    }

    fn with_source(id: FileId, source: Source) -> Self {
        let source_lock = OnceLock::new();
        let _ = source_lock.set(Ok(source));
        Self {
            id,
            source: Arc::new(source_lock),
            bytes: Arc::new(OnceLock::new()),
        }
    }

    fn source(&self, world: &WorldState) -> FileResult<Source> {
        self.source
            .get_or_init(|| {
                let path = world.get_physical_path(&self.id);
                let content =
                    std::fs::read_to_string(&path).map_err(|err| FileError::from_io(err, &path))?;

                Ok(Source::new(self.id.clone(), content))
            })
            .as_ref()
            .cloned()
            .map_err(|err| err.clone())
    }

    fn bytes(&self, world: &WorldState) -> FileResult<Bytes> {
        self.bytes
            .get_or_init(|| {
                let path = world.get_physical_path(&self.id);
                let content = std::fs::read(&path).map_err(|err| FileError::from_io(err, &path))?;

                Ok(Bytes::new(content))
            })
            .as_ref()
            .cloned()
            .map_err(|err| err.clone())
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct LoadError {
    pub error: String,
    pub path: String,
}

impl Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.path.is_empty() {
            write!(f, "Error loading typst file: {}", self.error)
        } else {
            write!(
                f,
                "Error loading typst file: {} (at {})",
                self.error, self.path
            )
        }
    }
}

impl Error for LoadError {}

#[derive(uniffi::Record)]
pub struct LoadResult {
    pub cards: Vec<CardInfo>,
    pub errors: Vec<LoadError>,
}

pub struct WorldState {
    /// Path of the cache directory
    cache_path: PathBuf,
    /// Client used for package downloading
    client: Client,
    /// Map of the loaded files, to avoid reading from fs every time.
    /// TODO: Expiration ?
    files: Mutex<HashMap<FileId, FileSlot>>,
    /// FileId of the virtual "_main.typ" source file (which doesn't actually exist on disk)
    main: FileId,
    /// FileId of the "_sha" file (which exists on disk), keeping track of the sha checksum of the
    /// latest commit to the cards repository, this avoids api spam.
    sha: FileId,
    /// List of newly created directories that need to be marked as group for android caching
    /// (see https://developer.android.com/reference/android/os/storage/StorageManager#setCacheBehaviorGroup(java.io.File,%20boolean) )
    new_directories: Mutex<Vec<PathBuf>>,
    /// Fuzzy matching
    /// Typst world stuff
    library: LazyHash<Library>,
    /// Typst world stuff
    book: LazyHash<FontBook>,
    /// Typst world stuff
    fonts: Vec<FontSlot>,
}

impl WorldState {
    pub fn new(cache_path: PathBuf) -> Self {
        let fonts = FontSearcher::new()
            .include_system_fonts(false)
            .include_embedded_fonts(true)
            .search();
        let mut font_book = fonts.book;
        let mut fonts = fonts.fonts.into_iter().map(FontSlot::from).collect_vec();

        for data in [
            include_bytes!("../assets/lexend_regular.ttf").as_slice(),
            include_bytes!("../assets/lexend_medium.ttf").as_slice(),
            include_bytes!("../assets/lexend_semibold.ttf").as_slice(),
            include_bytes!("../assets/lexend_bold.ttf").as_slice(),
            include_bytes!("../assets/notosansmath_regular.ttf").as_slice(),
        ] {
            let buffer = Bytes::new(data);
            for font in Font::iter(buffer) {
                font_book.push(font.info().clone());
                fonts.push(FontSlot::Extra(font));
            }
        }

        Self {
            book: LazyHash::new(font_book),
            cache_path,
            client: Client::new(),
            files: Mutex::new(HashMap::new()),
            fonts,
            library: LazyHash::new(Library::default()),
            main: FileId::new(None, VirtualPath::new("_main.typ")),
            new_directories: Mutex::new(Vec::new()),
            sha: FileId::new(None, VirtualPath::new("_sha")),
        }
    }
}

fn load_from_cache(core: &Core) -> Result<LoadResult, CoreError> {
    use crate::cards::CardCore;
    use rayon::prelude::*;

    // Collect entries first to enable parallel iteration
    let entries: Vec<_> = WalkDir::new(core.world.cache_path.join("workdir"))
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("typ"))
        .enumerate()
        .collect();

    let results: Vec<Result<Vec<CardInfo>, LoadError>> = entries
        .into_par_iter()
        .map(|(index, entry)| {
            let path_str = entry.path().to_string_lossy().to_string();
            let content = std::fs::read_to_string(entry.path()).map_err(|e| LoadError {
                error: e.to_string(),
                path: path_str.clone(),
            })?;

            core.parse(index as u64, &content).map_err(|e| LoadError {
                error: e.to_string(),
                path: path_str,
            })
        })
        .collect();

    let mut cards = Vec::new();
    let mut errors = Vec::new();

    for result in results {
        match result {
            Ok(mut items) => cards.append(&mut items),
            Err(e) => errors.push(e),
        }
    }

    Ok(LoadResult { cards, errors })
}

pub trait WorldCore {
    async fn load_from_github(
        &self,
        repo: String,
        branch: String,
        token: Option<String>,
    ) -> Result<LoadResult, CoreError>;
    fn prepare_source(
        &self,
        cards: impl IntoIterator<Item = Arc<dyn CardSource>>,
        config: SourceConfig,
    ) -> Result<(), CoreError>;
    fn inspect_source(&self) -> Option<String>;
    fn compile(&self) -> Result<Vec<Arc<CardPage>>, CoreError>;
    fn new_cached_directories(&self) -> Vec<PathBuf>;
}

impl WorldCore for Core {
    /// Return type is like that because we can get an error and recover
    async fn load_from_github(
        &self,
        repo: String,
        branch: String,
        token: Option<String>,
    ) -> Result<LoadResult, CoreError> {
        let api = GithubAPI::new(repo, branch, token).await?;
        let latest_sha = self.world.latest_sha();

        if latest_sha == api.sha {
            return load_from_cache(self);
        }

        let workdir = self.world.cache_path.join("workdir");

        if latest_sha != api.sha {
            // shas differ: we need to update our cache

            // Clear (workdir) cache
            if std::fs::exists(&workdir).unwrap_or_default() {
                std::fs::remove_dir_all(&workdir)?;
            }
            std::fs::create_dir_all(&workdir)?;

            let resp = api.get_tarball().await?.bytes().await?;
            let cursor = Cursor::new(resp);
            let decompressed = flate2::read::GzDecoder::new(cursor);
            let mut archive = tar::Archive::new(decompressed);

            for entry in archive.entries()? {
                let mut entry = entry?;
                let path = entry.path()?.to_path_buf();
                // Strip first component (root directory of the archive)
                let components: Vec<_> = path.components().collect();
                if components.len() > 1 {
                    let relative_path: PathBuf = components[1..].iter().collect();
                    entry.unpack(workdir.join(relative_path))?;
                }
            }

            self.world.save_sha(api.sha)?;
            self.world.new_directories.lock().push(workdir.clone());
        }

        load_from_cache(self)
    }
    fn prepare_source(
        &self,
        cards: impl IntoIterator<Item = Arc<dyn CardSource>>,
        config: SourceConfig,
    ) -> Result<(), CoreError> {
        use crate::cards::CardCore;

        let content = self.build_source(cards, config)?;

        self.world.files.lock().insert(
            self.world.main,
            FileSlot::with_source(self.world.main, Source::new(self.world.main, content)),
        );

        Ok(())
    }
    fn compile(&self) -> Result<Vec<Arc<CardPage>>, CoreError> {
        let output = typst::compile::<PagedDocument>(&self.world)
            .output
            .map_err(|errors| CoreError::Typst {
                details: format!("{errors:?}"),
            })?;

        Ok(output
            .pages
            .into_iter()
            .map(|p| Arc::new(CardPage(p)))
            .collect_vec())
    }
    fn inspect_source(&self) -> Option<String> {
        Some(
            self.world
                .get_file(&self.world.main)?
                .source
                .get()?
                .as_ref()
                .ok()?
                .text()
                .to_string(),
        )
    }
    fn new_cached_directories(&self) -> Vec<PathBuf> {
        self.world.new_directories.lock().drain(..).collect_vec()
    }
}

impl WorldState {
    /// Get the directory associated with a PackageSpec
    fn get_package_directory(&self, spec: &PackageSpec) -> PathBuf {
        self.cache_path
            .join("packages")
            .join(format!("{}/{}/{}", spec.namespace, spec.name, spec.version))
    }

    /// Get the physical path associated with a FileId
    fn get_physical_path(&self, id: &FileId) -> PathBuf {
        match id.package() {
            None => self
                .cache_path
                .join("workdir")
                .join(id.vpath().as_rootless_path()),
            Some(spec) => self
                .get_package_directory(spec)
                .join(id.vpath().as_rootless_path()),
        }
    }

    /// Add a file to the loaded files map (may or may not exist on disk)
    fn discover_file(&self, id: FileId, slot: FileSlot) -> Option<FileSlot> {
        self.files.lock().insert(id, slot)
    }

    /// Get a file, loading it if not currently loaded (but exists on disk)
    fn get_file(&self, id: &FileId) -> Option<FileSlot> {
        if let Some(slot) = self.files.lock().get(id).cloned() {
            return Some(slot);
        }

        let path = self.get_physical_path(id);

        if !std::fs::exists(&path).ok()? {
            return None;
        }

        let slot = FileSlot::new(id.clone());

        self.discover_file(id.clone(), slot.clone());

        Some(slot)
    }

    /// Write to a file with a given path, creating all parent directories if needed
    fn write_file(&self, content: String, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let path = path.as_ref();
        let prefix = path.parent().unwrap();

        std::fs::create_dir_all(prefix)?;
        std::fs::write(path, content)
    }

    /// Read disk to find the sha of cached workdir
    fn latest_sha(&self) -> String {
        self.get_file(&self.sha)
            .and_then(|s| s.source(&self).ok())
            .map(|s| s.text().to_string())
            .unwrap_or_default()
    }

    /// Save a new sha to disk
    fn save_sha(&self, sha: String) -> Result<(), std::io::Error> {
        self.write_file(sha, self.get_physical_path(&self.sha))
    }

    /// Download package to cache
    fn download_package(&self, spec: &PackageSpec) -> Result<(), std::io::Error> {
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

        let decompressed = flate2::read::GzDecoder::new(&data[..]);
        tar::Archive::new(decompressed)
            .unpack(&dir)
            .map_err(|err| {
                fs::remove_dir_all(&dir).ok();
                std::io::Error::new(std::io::ErrorKind::Other, err)
            })?;

        self.new_directories.lock().push(dir);

        Ok(())
    }
}

/// Newtype around typst::layout::Page because I need it to derive uniffi::Object
#[derive(uniffi::Object)]
pub struct CardPage(Page);

#[uniffi::export]
impl CardPage {
    pub fn svg(&self) -> String {
        typst_svg::svg(&self.0)
    }
}

impl TypstWorld for WorldState {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }
    fn main(&self) -> FileId {
        self.main
    }
    fn source(&self, id: FileId) -> FileResult<Source> {
        if let Some(slot) = self.get_file(&id) {
            slot.source(&self)
        } else if let Some(spec) = id.package() {
            // Try to download the package
            self.download_package(spec)
                .map_err(|err| FileError::from_io(err, &self.get_physical_path(&id)))?;

            self.get_file(&id)
                .ok_or_else(|| FileError::NotFound(self.get_physical_path(&id)))
                .and_then(|slot| slot.source(&self))
        } else {
            Err(FileError::AccessDenied)
        }
    }
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if let Some(slot) = self.get_file(&id) {
            slot.bytes(&self)
        } else if let Some(spec) = id.package() {
            // Try to download the package
            self.download_package(spec)
                .map_err(|err| FileError::from_io(err, &self.get_physical_path(&id)))?;

            self.get_file(&id)
                .ok_or_else(|| FileError::NotFound(self.get_physical_path(&id)))
                .and_then(|slot| slot.bytes(&self))
        } else {
            Err(FileError::AccessDenied)
        }
    }
    fn font(&self, index: usize) -> Option<Font> {
        Some(self.fonts.get(index)?.get()?)
    }
    fn today(&self, _offset: Option<i64>) -> Option<typst::foundations::Datetime> {
        None
    }
}
