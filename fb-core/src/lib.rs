use std::{
    fmt::Debug,
    path::PathBuf,
    sync::Arc,
};

use android_logger::{Config, FilterBuilder};

use crate::{
    cards::{CardSource, CardState, SourceConfig},
    fuzzy::{FuzzyCore, FuzzyItem, FuzzyState, FuzzyStatus},
    world::{CardPage, LoadResult, WorldCore, WorldState},
};

uniffi::setup_scaffolding!();

mod cards;
mod fsrs;
mod fuzzy;
mod github;
mod world;

/// Main struct of this library, acts as a global context for the different features this
/// implements
#[derive(uniffi::Object)]
pub struct Core {
    pub card: CardState,
    pub world: WorldState,
    pub fuzzy: FuzzyState,
}

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum CoreError {
    #[error("couldn't parse typst file for cards: {message}")]
    Parsing { message: String },
    #[error("IO error: {message}")]
    IO { message: String },
    #[error("Http (Reqwest) error: {message}")]
    HTTP { message: String },
    #[error("Typst error: {message}")]
    Typst { message: String },
}

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for CoreError {
    fn from(value: nom::Err<nom::error::Error<&'a str>>) -> Self {
        Self::Parsing {
            message: match value {
                nom::Err::Incomplete(needed) => match needed {
                    nom::Needed::Unknown => format!("Incomplete input"),
                    nom::Needed::Size(size) => format!("Incomplete input, missing ({size}) bytes"),
                },
                nom::Err::Error(err) => format!("Error while parsing: {err} ({err:?})"),
                nom::Err::Failure(fail) => format!("Failed to parse: {fail} ({fail:?})"),
            },
        }
    }
}

impl From<std::io::Error> for CoreError {
    fn from(value: std::io::Error) -> Self {
        Self::IO {
            message: format!("{value} ({value:?})"),
        }
    }
}

impl From<reqwest::Error> for CoreError {
    fn from(value: reqwest::Error) -> Self {
        Self::HTTP {
            message: format!("{value} ({value:?})"),
        }
    }
}

impl Core {
    pub fn new(cache_path: PathBuf) -> Self {
        Self {
            card: CardState::new(),
            world: WorldState::new(cache_path),
            fuzzy: FuzzyState::new(),
        }
    }
}

#[allow(non_snake_case)]
#[uniffi::export]
impl Core {
    fn worldLoadFromGithub(
        &self,
        repo: String,
        branch: String,
        token: Option<String>,
    ) -> Result<LoadResult, CoreError> {
        WorldCore::load_from_github(self, repo, branch, token)
    }
    fn worldPrepareSource(
        &self,
        cards: Vec<Arc<dyn CardSource>>,
        config: SourceConfig,
    ) -> Result<(), CoreError> {
        WorldCore::prepare_source(self, cards, config)
    }
    fn worldCompile(&self) -> Result<Vec<Arc<CardPage>>, CoreError> {
        WorldCore::compile(self)
    }
    fn fuzzyInit(&self, pattern: String) {
        FuzzyCore::init(self, &pattern);
    }
    fn fuzzyTick(&self) -> FuzzyStatus {
        FuzzyCore::tick(self)
    }
    fn fuzzyResults(&self) -> Vec<Arc<dyn FuzzyItem>> {
        FuzzyCore::results(self)
    }
    fn fuzzyAddItem(&self, item: Arc<dyn FuzzyItem>) {
        FuzzyCore::add_item(self, item);
    }
    fn fuzzyReset(&self) {
        FuzzyCore::reset(self);
    }
}

#[uniffi::export]
#[uniffi::method(name = "rustSetupLogger")]
pub fn rust_setup_logger() {
    android_logger::init_once(
        Config::default()
            .with_max_level(log::LevelFilter::Trace)
            .with_filter(
                FilterBuilder::new()
                    .parse("info,fb-core::crate=trace")
                    .build(),
            )
            .with_tag("fb-core"),
    );
}
