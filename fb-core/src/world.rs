use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use itertools::Itertools;
use log::{trace, warn};
use parking_lot::Mutex;
use reqwest::{StatusCode, blocking::Client};
use typst::{
    Library, World as TypstWorld,
    diag::{FileError, FileResult},
    foundations::Bytes,
    layout::{Page, PagedDocument},
    syntax::{FileId, Source, VirtualPath, package::PackageSpec},
    text::{Font, FontBook},
    utils::LazyHash,
};
use typst_kit::fonts::{FontSearcher, FontSlot};
use walkdir::WalkDir;

use crate::{
    cards::{Card, CardStore, FuzzyStatus, SourceConfig, Tag},
    github::GithubAPI, studies::{Study, StudyState, StudyStore},
};

/// The default Typst registry.
pub const DEFAULT_REGISTRY: &str = "https://packages.typst.org";

/// The public namespace in the default Typst registry.
pub const DEFAULT_NAMESPACE: &str = "preview";

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

    fn source(&self, world: &World) -> FileResult<Source> {
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

    fn bytes(&self, world: &World) -> FileResult<Bytes> {
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
    error: String,
    path: String,
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

#[derive(uniffi::Object)]
pub struct World {
    /// Source config used for cards source building
    config: Mutex<SourceConfig>,
    cards: Mutex<CardStore>,
    studies: Mutex<StudyStore>,
    /// Path of the cache directory
    cache_path: PathBuf,
    /// List of newly created directories that need to be marked as group for android caching
    /// (see https://developer.android.com/reference/android/os/storage/StorageManager#setCacheBehaviorGroup(java.io.File,%20boolean) )
    new_directories: Mutex<Vec<PathBuf>>,
    /// Map of the loaded files, to avoid reading from fs every time.
    /// TODO: Expiration ?
    files: Mutex<HashMap<FileId, FileSlot>>,
    /// FileId of the virtual "_main.typ" source file (which doesn't actually exist on disk)
    main: FileId,
    /// FileId of the "_sha" file (which exists on disk), keeping track of the sha checksum of the
    /// latest commit to the cards repository, this avoids api spam.
    sha: FileId,
    /// List of the cards that need to be rendered
    selected_cards: Mutex<Vec<Card>>,
    /// Client used for package downloading
    client: Client,

    // Typst stuff
    library: LazyHash<Library>,
    fonts: Vec<FontSlot>,
    book: LazyHash<FontBook>,
}

impl World {
    /// Create a new empty world, this should probably be followed by a load_from_github call to
    /// fill the world
    pub fn empty(cache_path: PathBuf, config: SourceConfig) -> Self {
        let library = LazyHash::new(Library::default());
        let fonts = FontSearcher::new()
            .include_system_fonts(false)
            .include_embedded_fonts(true)
            .search();
        let book = LazyHash::new(fonts.book);
        let fonts = fonts.fonts;
        let cards = Mutex::new(CardStore::default());
        let studies = Mutex::new(StudyStore::default());
        let files = Mutex::new(HashMap::new());
        let selected_cards = Mutex::new(Vec::new());
        let main = FileId::new(None, VirtualPath::new("_main.typ"));
        let sha = FileId::new(None, VirtualPath::new("_sha"));
        let new_directories = Mutex::new(Vec::new());
        let client = Client::new();
        let config = Mutex::new(config);

        Self {
            client,
            config,
            main,
            sha,
            cards,
            studies,
            cache_path,
            library,
            fonts,
            book,
            files,
            selected_cards,
            new_directories,
        }
    }

    /// Get the directory associated with a PackageSpec
    fn get_package_directory(&self, spec: &PackageSpec) -> PathBuf {
        self.cache_path.join("packages").join(format!(
            "{}/{}/{}",
            spec.namespace, spec.name, spec.version
        ))
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
        warn!("Getting file {id:?}");

        if let Some(slot) = self.files.lock().get(id).cloned() {
            warn!("File already loaded :D");
            return Some(slot);
        }

        let path = self.get_physical_path(id);
        warn!("Loading file... {path:?}");

        if !std::fs::exists(&path).ok()? {
            warn!("File doesn't exist D:");
            return None;
        }

        let slot = FileSlot::new(id.clone());

        warn!("Caching file as loaded");
        self.discover_file(id.clone(), slot.clone());

        Some(slot)
    }

    /// Write to a file with a given path, creating all parent directories if needed
    fn write_file(&self, content: String, path: impl AsRef<Path>) -> Result<(), std::io::Error> {
        let path = path.as_ref();
        let prefix = path.parent().unwrap();

        warn!("HELLO PLS HEAR ME : {path:?} {prefix:?}, I'm going to try to create the directories");
        std::fs::create_dir_all(prefix)?;
        warn!("I DID IT ! About to write the file now");
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
    pub fn download_package(&self, spec: &PackageSpec) -> Result<(), std::io::Error> {
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

    /// Return type is like that because we can get an error and recover
    pub fn load_from_github(
        &self,
        repo: String,
        branch: String,
        token: Option<String>,
    ) -> Result<Vec<LoadError>, Box<dyn Error>> {
        trace!("Begining");

        let api = GithubAPI::new(repo, branch, token)?;
        let mut errors = Vec::new();

        trace!("Initilization done, clearing store");

        let latest_sha = self.latest_sha();

        if latest_sha == api.sha {
            trace!("SHAs match ! I will load all the files I can");

            self.cards.lock().mark_cards_for_garbage_collection();

            // We can use cached data
            for entry in WalkDir::new(self.cache_path.join("workdir"))
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("typ"))
            {
                let content = std::fs::read_to_string(entry.path())?;
                if let Err(e) = self.cards.lock().load(&content) {
                    errors.push(LoadError {
                        error: e.to_string(),
                        path: entry.path().to_string_lossy().to_string(),
                    });
                };
            }

            self.cards.lock().sha = latest_sha;
            self.cards.lock().collect_garbage();

            return Ok(errors);
        }

        trace!("SHAs differ");

        // shas differ: we need to updated our cache

        let workdir = self.cache_path.join("workdir");
        // Clear (workdir) cache
        if std::fs::exists(&workdir).unwrap_or_default() {
            trace!("Clearing previous workdir {workdir:?}");
            std::fs::remove_dir_all(&workdir)?;
        }

        let items = api.get_items()?.into_iter().filter(|item| {
            item.kind == "blob"
                && Path::new(&item.path)
                    .extension()
                    .map(|ext| ext == "typ")
                    .unwrap_or_default()
        });

        trace!("Going to process all repository items");

        self.cards.lock().mark_cards_for_garbage_collection();

        for item in items {
            trace!("Item: {}", &item.path);

            let content = api.get_blob(&item.sha)?;

            if let Err(e) = self.cards.lock().load(&content) {
                errors.push(LoadError {
                    error: e.to_string(),
                    path: item.path.clone(),
                });
            };


            let file_id = FileId::new(None, VirtualPath::new(&item.path));
            trace!("Installing: {} -> {:?}", item.path, self.get_physical_path(&file_id));
            self.write_file(content, self.get_physical_path(&file_id))?;
        }

        trace!("Done, saving SHA");

        self.cards.lock().collect_garbage();
        self.cards.lock().sha = api.sha.clone();
        self.save_sha(api.sha)?;

        self.new_directories.lock().push(workdir);

        trace!("returning");

        Ok(errors)
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

#[derive(uniffi::Object)]
pub struct AnyError {
    display: String,
    debug: String,
}

impl Debug for AnyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.debug)
    }
}

impl Display for AnyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display)
    }
}

#[uniffi::export]
impl AnyError {
    #[uniffi::method(name = "display")]
    pub fn _display(&self) -> String {
        self.display.clone()   
    }
    #[uniffi::method(name = "debug")]
    pub fn _debug(&self) -> String {
        self.debug.clone()
    }
}

impl Error for AnyError {}

impl AnyError {
    fn from_error(value: Box<dyn Error>) -> Self {
        Self {
            debug: format!("{value:?}"),
            display: format!("{value}"),
        }
    }
}


impl TypstWorld for World {
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
        if id == self.main {
            match self
                .cards
                .lock()
                .build_source(self.selected_cards.lock().iter().cloned(), *self.config.lock())
            {
                Ok(content) => Ok(Source::new(id, content)),
                Err(err) => {
                    return Err(FileError::from_io(err, &self.get_physical_path(&self.main)));
                }
            }
        } else if let Some(slot) = self.get_file(&id) {
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

#[uniffi::export]
impl World {
    #[uniffi::constructor]
    fn _empty(cache_path: String, config: SourceConfig) -> Self {
        Self::empty(PathBuf::from(&cache_path), config)
    }

    #[uniffi::method(name = "setSourceConfig")]
    fn _set_source_config(&self, config: SourceConfig) {
        *self.config.lock() = config
    }

    #[uniffi::method(name = "setSelectedCards")]
    fn _set_selected_cards(&self, cards: Vec<Card>) {
        *self.selected_cards.lock() = cards;
    }

    #[uniffi::method(name = "loadFromGithub")]
    fn _load_from_github(
        &self,
        repo: String,
        branch: String,
        token: Option<String>,
    ) -> Result<Vec<LoadError>, AnyError> {
        self.load_from_github(repo, branch, token)
            .map_err(AnyError::from_error)
    }

    #[uniffi::method(name = "compile")]
    fn _compile(&self) -> Result<Vec<Arc<CardPage>>, AnyError> {
        let output = typst::compile::<PagedDocument>(&self)
            .output
            .map_err(|errors| {
                let str = format!("{errors:?}");
                AnyError {
                    display: str.to_string(),
                    debug: str.to_string(),
                }
            })?;

        Ok(output
            .pages
            .into_iter()
            .map(|p| Arc::new(CardPage(p)))
            .collect_vec())
    }

    #[uniffi::method(name = "newCachedDirectories")]
    fn _new_cached_directories(&self) -> Vec<String> {
        self.new_directories
            .lock()
            .drain(..)
            .map(|p| p.to_string_lossy().to_string())
            .collect_vec()
    }

    #[uniffi::method(name = "cards")]
    fn _cards(&self) -> Vec<Card> {
        self.cards.lock().cards()
    }

    #[uniffi::method(name = "roots")]
    fn _roots(&self) -> Vec<Tag> {
        self.cards.lock().roots()
    }

    #[uniffi::method(name = "fuzzyInit")]
    fn _fuzzy_init(&self, pattern: String) {
        self.cards.lock().fuzzy_init(&pattern);
    }

    #[uniffi::method(name = "fuzzyTick")]
    fn _fuzzy_tick(&self) -> FuzzyStatus {
        self.cards.lock().fuzzy_tick()
    }

    #[uniffi::method(name = "fuzzyResults")]
    fn _fuzzy_results(&self) -> Vec<Card> {
        self.cards.lock().fuzzy_results()
    }

    #[uniffi::method(name = "loadStudy")]
    fn _load_study(&self, id: u64, timestamp: u64, selection: Vec<Card>, state: StudyState) -> Study {
        self.studies.lock().load_study(id, timestamp, selection, state)
    }

    #[uniffi::method(name = "newStudy")]
    fn _new_study(&self, name: String, selection: Vec<Card>) -> Study {
        self.studies.lock().new_study(name, selection)
    }

    #[uniffi::method(name = "getStudies")]
    fn _get_studies(&self) -> Vec<Study> {
        self.studies.lock().studies().collect_vec()
    }

    #[uniffi::method(name = "studyLastId")]
    fn _study_last_id(&self) -> Option<u64> {
        self.studies.lock().last_id()
    }

    #[uniffi::method(name = "studySetLastId")]
    fn _study_set_last_id(&self, value: u64) {
        self.studies.lock().set_last_id(value)
    }

    #[uniffi::method(name = "deleteStudy")]
    fn _delete_study(&self, id: u64) -> Option<Study> {
        self.studies.lock().delete_study(id)
    }
}
