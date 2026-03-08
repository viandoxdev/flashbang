use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    io::{Cursor, Read},
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
};

use itertools::Itertools;
use parking_lot::Mutex;
use typst::{
    Library, LibraryExt, World as TypstWorld,
    diag::{FileError, FileResult},
    foundations::Bytes,
    layout::{Page, PagedDocument},
    syntax::{FileId, Source, VirtualPath},
    text::{Font, FontBook},
    utils::LazyHash,
};
use typst_kit::fonts::{FontSearcher, FontSlot as TypstFontSlot};

#[cfg(feature = "cache")]
use crate::cache::CacheProvider;
use crate::{
    cards::{CardInfo, CardSource, CardState, SourceConfig},
    error::CoreError,
    packages::PackageProvider,
};

#[cfg(feature = "github")]
use crate::github::GithubAPI;

trait StripFirstComponent {
    fn pop_front<'a>(&'a self) -> &'a Path;
}

impl StripFirstComponent for Path {
    fn pop_front(&self) -> &Path {
        let mut components = self.components();
        components.next();
        components.as_path()
    }
}

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

// TODO: Simplify this as the API has changed when abstracting the filesystem away
// Make it a simple enum with an id function or something i don't know

#[derive(Debug, Clone)]
pub struct FileSlot {
    id: FileId,
    source: Arc<Option<FileResult<Source>>>,
    bytes: Arc<Option<FileResult<Bytes>>>,
}

impl FileSlot {
    pub fn with_source(id: FileId, source: Source) -> Self {
        Self {
            id,
            source: Arc::new(Some(Ok(source))),
            bytes: Arc::new(None),
        }
    }

    pub fn with_bytes(id: FileId, bytes: Bytes) -> Self {
        Self {
            id,
            source: Arc::new(None),
            bytes: Arc::new(Some(Ok(bytes))),
        }
    }

    /// Virtual path used for error messages
    fn debug_path(&self) -> PathBuf {
        match self.id.package() {
            Some(spec) => PathBuf::from(format!(
                "packages/{}/{}/{}",
                spec.namespace, spec.name, spec.version
            )),
            None => PathBuf::from_str("local").unwrap(),
        }
        .join(self.id.vpath().as_rootless_path())
    }

    pub fn source(&self) -> FileResult<Source> {
        match self.source.as_ref() {
            Some(res) => res.clone(),
            None => Err(FileError::NotFound(self.debug_path())),
        }
    }

    pub fn bytes(&self) -> FileResult<Bytes> {
        match self.bytes.as_ref() {
            Some(res) => res.clone(),
            None => Err(FileError::NotFound(self.debug_path())),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
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

trait LoadErrorContext {
    type T;
    fn context(self, path: Option<&PathBuf>) -> Result<Self::T, LoadError>;
}

impl<T, E: Error> LoadErrorContext for Result<T, E> {
    type T = T;
    fn context(self, path: Option<&PathBuf>) -> Result<Self::T, LoadError> {
        let path = match path {
            Some(path) => path.to_string_lossy().to_string(),
            None => "<nowhere>".to_string(),
        };

        match self {
            Ok(v) => Ok(v),
            Err(e) => Err(LoadError {
                error: e.to_string(),
                path,
            }),
        }
    }
}

impl Error for LoadError {}

#[cfg_attr(feature = "uniffi", derive(uniffi::Record))]
pub struct LoadResult {
    cards: Vec<CardInfo>,
    errors: Vec<LoadError>,
}

pub struct WorldState {
    /// Map of the loaded files, to avoid reading from fs every time.
    /// TODO: Expiration ?
    files: Mutex<HashMap<FileId, FileSlot>>,
    /// FileId of the "_main.typ" source file
    main: FileId,
    /// Fuzzy matching
    /// Typst world stuff
    library: LazyHash<Library>,
    /// Typst world stuff
    book: LazyHash<FontBook>,
    /// Typst world stuff
    fonts: Vec<FontSlot>,
    packages: Box<dyn PackageProvider>,
    /// Cache abstraction
    #[cfg(feature = "cache")]
    cache: Box<dyn CacheProvider>,
}

impl WorldState {
    pub fn new(
        package_provider: impl PackageProvider,
        #[cfg(feature = "cache")] cache_provider: impl CacheProvider,
    ) -> Self {
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
            packages: Box::new(package_provider),
            book: LazyHash::new(font_book),
            files: Mutex::new(HashMap::new()),
            fonts,
            library: LazyHash::new(Library::default()),
            main: FileId::new(None, VirtualPath::new("_main.typ")),
            #[cfg(feature = "cache")]
            cache: Box::new(cache_provider),
        }
    }
}

fn load_from_tarball(
    world: &WorldState,
    cards: &CardState,
    tarball: impl Read,
) -> Result<LoadResult, CoreError> {
    let decompressed = flate2::read::GzDecoder::new(tarball);
    let mut archive = tar::Archive::new(decompressed);

    let results = archive
        .entries()?
        .enumerate()
        .map(|(id, entry)| -> Result<_, LoadError> {
            let entry = entry.context(None)?;
            let path = entry.path().context(None)?.to_path_buf();

            Ok((id as u64, entry, path))
        })
        .filter_ok(|(_, _, path)| path.extension().and_then(|ext| ext.to_str()) == Some("typ"))
        .map(|entry| {
            entry.and_then(|(id, mut entry, path)| {
                let mut content = String::new();
                entry.read_to_string(&mut content).context(Some(&path))?;

                if content.starts_with("//![FLASHBANG INCLUDE]") {
                    let file_id = FileId::new(None, VirtualPath::new(path.pop_front()));
                    world.load_file(FileSlot::with_source(
                        file_id,
                        Source::new(file_id, content),
                    ));

                    Ok(Vec::new())
                } else {
                    cards.parse(id, &content).context(Some(&path))
                }
            })
        });

    let mut load_res = LoadResult {
        cards: Vec::new(),
        errors: Vec::new(),
    };

    for res in results {
        match res {
            Ok(mut cards) => load_res.cards.append(&mut cards),
            Err(error) => load_res.errors.push(error),
        }
    }

    Ok(load_res)
}

impl WorldState {
    // Return type is like that because we can get an error and recover
    #[cfg(all(feature = "github", feature = "cache"))]
    pub fn load_from_github(
        &self,
        cards: &CardState,
        repo: String,
        branch: String,
        token: Option<String>,
    ) -> Result<LoadResult, CoreError> {
        let api = GithubAPI::new(repo, branch, token)?;
        let latest_sha = self.cache.get_sha().unwrap_or_default();

        if latest_sha == api.sha
            && let Ok(tarball) = self.cache.get_tarball()
        {
            // Shas match and cache is working, use that
            return Ok(load_from_tarball(self, cards, tarball)?);
        }

        // shas differ or cache is broken, we need to update our cache
        let tarball = api.get_tarball()?.bytes()?;

        self.cache.save_tarball(&mut Cursor::new(tarball.clone()))?;
        self.cache.save_sha(api.sha)?;

        Ok(load_from_tarball(self, cards, Cursor::new(tarball))?)
    }

    #[cfg(all(feature = "github", not(feature = "cache")))]
    pub fn load_from_github(
        &self,
        cards: &CardState,
        repo: String,
        branch: String,
        token: Option<String>,
    ) -> Result<LoadResult, CoreError> {
        let api = GithubAPI::new(repo, branch, token)?;
        let tarball = api.get_tarball()?;
        load_from_tarball(cards, tarball)
    }

    pub fn prepare_source(
        &self,
        cards: &CardState,
        items: impl IntoIterator<Item = Arc<dyn CardSource>>,
        config: SourceConfig,
    ) -> Result<(), CoreError> {
        let content = cards.build_source(items, config)?;

        self.files.lock().insert(
            self.main,
            FileSlot::with_source(self.main, Source::new(self.main, content)),
        );

        Ok(())
    }
    pub fn compile(&self) -> Result<Vec<Arc<CardPage>>, CoreError> {
        let output = typst::compile::<PagedDocument>(&self)
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
    pub fn inspect_source(&self) -> Option<String> {
        Some(self.get_file(&self.main)?.source().ok()?.text().to_string())
    }
}

impl WorldState {
    /// Add a file to the loaded files map
    pub fn load_file(&self, slot: FileSlot) -> Option<FileSlot> {
        self.files.lock().insert(slot.id, slot)
    }

    /// Get a file
    pub fn get_file(&self, id: &FileId) -> Option<FileSlot> {
        self.files.lock().get(id).cloned()
    }
}

/// Newtype around typst::layout::Page because I need it to derive uniffi::Object
#[cfg_attr(feature = "uniffi", derive(uniffi::Object))]
pub struct CardPage(Page);

#[cfg_attr(feature = "uniffi", uniffi::export)]
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
        if id.package().is_some() {
            self.packages.get_package_source(id, &self)
        } else if let Some(slot) = self.get_file(&id) {
            slot.source()
        } else {
            Err(FileError::AccessDenied)
        }
    }
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id.package().is_some() {
            self.packages.get_package_file(id, &self)
        } else if let Some(slot) = self.get_file(&id) {
            slot.bytes()
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
