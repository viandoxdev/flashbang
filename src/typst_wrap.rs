use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use typst::{
    diag::{FileError, FileResult},
    foundations::Datetime,
    syntax::{FileId, Source, VirtualPath},
    text::{Font, FontBook},
    utils::LazyHash,
    Library,
};
use typst_kit::fonts::{FontSearcher, FontSlot};

pub struct TypstWrapper {
    root: PathBuf,
    files: HashMap<FileId, Source>,
    library: LazyHash<Library>,
    fonts: Vec<FontSlot>,
    book: LazyHash<FontBook>,
    source: FileId,
}

impl TypstWrapper {
    pub fn new(root: impl AsRef<Path>, source: impl Into<String>) -> Self {
        let fonts = FontSearcher::new()
            .include_system_fonts(false)
            .include_embedded_fonts(true)
            .search();
        let mut files = HashMap::new();
        let mut add_file = |path, content| {
            let id = FileId::new(None, VirtualPath::new(path));
            let source = Source::new(id, content);
            files.insert(id, source);
            id
        };

        let source = add_file("main.typ", source.into());
        add_file(
            "cards_internal.typ",
            include_str!("../cards_internal.typ").to_string(),
        );

        Self {
            root: root.as_ref().to_owned(),
            book: LazyHash::new(fonts.book),
            fonts: fonts.fonts,
            library: LazyHash::new(Library::default()),
            files,
            source,
        }
    }
}

impl typst::World for TypstWrapper {
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }
    fn file(&self, _id: typst::syntax::FileId) -> FileResult<typst::foundations::Bytes> {
        FileResult::Err(FileError::AccessDenied)
    }
    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).and_then(|s| s.get())
    }
    fn main(&self) -> typst::syntax::FileId {
        self.source
    }
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        None
    }
    fn source(&self, id: typst::syntax::FileId) -> FileResult<Source> {
        self.files
            .get(&id)
            .cloned()
            .ok_or_else(|| FileError::AccessDenied)
    }
}
