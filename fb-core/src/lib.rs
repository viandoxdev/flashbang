use std::{path::PathBuf, sync::Arc};

use android_logger::{Config, FilterBuilder};
use fsrs::MemoryState;
use itertools::Itertools;

use crate::{
    cards::{CardSource, CardState, SourceConfig},
    error::CoreError,
    fuzzy::{FuzzyCore, FuzzyItem, FuzzyState, FuzzyStatus},
    scheduler::{
        Progress, SchedulerCore, SchedulerItem, SchedulerMemoryState, SchedulerNextState,
        SchedulerState,
    },
    world::{CardPage, LoadResult, WorldCore, WorldState},
};

uniffi::setup_scaffolding!();

mod cards;
mod error;
mod fuzzy;
mod github;
mod scheduler;
mod world;

/// Main struct of this library, acts as a global context for the different features this
/// implements
#[derive(uniffi::Object)]
pub struct Core {
    pub card: CardState,
    pub world: WorldState,
    pub fuzzy: FuzzyState,
    pub scheduler: SchedulerState,
}

impl Core {
    pub fn new(cache_path: PathBuf) -> Result<Self, CoreError> {
        Ok(Self {
            card: CardState::new(),
            world: WorldState::new(cache_path),
            fuzzy: FuzzyState::new(),
            scheduler: SchedulerState::new()?,
        })
    }
}

#[allow(non_snake_case)]
#[uniffi::export]
impl Core {
    #[uniffi::constructor]
    fn _new(cache_path: String) -> Result<Self, CoreError> {
        Self::new(PathBuf::from(&cache_path))
    }
    fn worldLoadFromGithub(
        &self,
        repo: String,
        branch: String,
        token: Option<String>,
    ) -> Result<LoadResult, CoreError> {
        WorldCore::load_from_github(self, repo, branch, token)
    }
    fn worldInspectSource(&self) -> Option<String> {
        WorldCore::inspect_source(self)
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
    fn worldNewCachedDirectories(&self) -> Vec<String> {
        WorldCore::new_cached_directories(self)
            .into_iter()
            .map(|path| path.to_string_lossy().to_string())
            .collect_vec()
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
    fn schedulerSetParameters(&self, parameters: Vec<f32>) -> Result<(), CoreError> {
        SchedulerCore::set_parameters(self, &parameters)
    }
    fn schedulerNextState(
        &self,
        state: Option<SchedulerMemoryState>,
        days_elapsed: u32,
    ) -> Result<SchedulerNextState, CoreError> {
        Ok(SchedulerNextState::from(SchedulerCore::next_state(
            self,
            state.map(MemoryState::from),
            days_elapsed,
        )?))
    }
    // We have to arc proress (even though it already contains an arc) for ffi
    fn schedulerRecomputeParameters(
        &self,
        items: Vec<SchedulerItem>,
        progress: Option<Arc<Progress>>,
    ) -> Result<Vec<f32>, CoreError> {
        SchedulerCore::compute_parameters(
            self,
            items.into_iter().map(From::from).collect_vec(),
            progress.map(|v| <Progress as Clone>::clone(&*v)),
        )
    }
    fn schedulerSetRetention(&self, value: f32) {
        SchedulerCore::set_retention(self, value);
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
