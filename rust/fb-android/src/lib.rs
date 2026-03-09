use std::{path::PathBuf, sync::Arc};

use android_logger::{Config, FilterBuilder};
use fb_core::{
    cards::{CardSource, CardState, SourceConfig},
    error::CoreError,
    fsrs::MemoryState,
    fuzzy::{FuzzyItem, FuzzyState, FuzzyStatus},
    scheduler::{
        Progress, SchedulerItem, SchedulerMemoryState, SchedulerNextState, SchedulerState,
    },
    world::{CardPage, LoadResult, WorldState},
};
use parking_lot::Mutex;

use crate::{
    cache_provider::FileSystemCacheProvider, package_provider::DownloadingPackageProvider,
};

mod cache_provider;
mod package_provider;

uniffi::setup_scaffolding!();

/// Main struct of this library, acts as a global context for the different features this
/// implements
#[derive(uniffi::Object)]
pub struct Core {
    card: CardState,
    world: WorldState,
    fuzzy: FuzzyState,
    scheduler: SchedulerState,
    cache_groups: Arc<Mutex<Vec<String>>>,
}

impl Core {
    pub fn new(data_path: PathBuf) -> Result<Self, CoreError> {
        let packages_path = data_path.join("packages");
        let cache_path = data_path.join("repo");

        let cache_groups = Arc::new(Mutex::new(vec![cache_path.to_string_lossy().to_string()]));
        let package_provider = DownloadingPackageProvider::new(packages_path, cache_groups.clone());
        let cache_provider = FileSystemCacheProvider::new(cache_path);

        Ok(Self {
            card: CardState::new(),
            world: WorldState::new(package_provider, cache_provider),
            fuzzy: FuzzyState::new(),
            scheduler: SchedulerState::new()?,
            cache_groups,
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
        self.world.load_from_github(&self.card, repo, branch, token)
    }
    fn worldInspectSource(&self) -> Option<String> {
        self.world.inspect_source()
    }
    fn worldPrepareSource(
        &self,
        cards: Vec<Arc<dyn CardSource>>,
        config: SourceConfig,
    ) -> Result<(), CoreError> {
        self.world.prepare_source(&self.card, cards, config)
    }
    fn worldCompile(&self) -> Result<Vec<Arc<CardPage>>, CoreError> {
        self.world
            .compile()
            .map(|pages| pages.into_iter().map(|p| Arc::new(p)).collect())
    }
    fn fuzzyInit(&self, pattern: String) {
        self.fuzzy.init(&pattern);
    }
    fn fuzzyTick(&self) -> FuzzyStatus {
        self.fuzzy.tick()
    }
    fn fuzzyResults(&self) -> Vec<Arc<dyn FuzzyItem>> {
        self.fuzzy
            .with_results(|iter| iter.map(|i| i.data.clone()).collect())
    }
    fn fuzzyAddItem(&self, item: Arc<dyn FuzzyItem>) {
        self.fuzzy.add_item(item);
    }
    fn fuzzyAddItems(&self, items: Vec<Arc<dyn FuzzyItem>>) {
        self.fuzzy.add_items(items);
    }
    fn fuzzyReset(&self) {
        self.fuzzy.reset();
    }
    fn schedulerSetParameters(&self, parameters: Vec<f32>) -> Result<(), CoreError> {
        self.scheduler.set_parameters(&parameters)
    }
    fn worldNewCachedDirectories(&self) -> Vec<String> {
        self.cache_groups.lock().clone()
    }
    fn schedulerNextState(
        &self,
        state: Option<SchedulerMemoryState>,
        days_elapsed: u32,
    ) -> Result<SchedulerNextState, CoreError> {
        Ok(SchedulerNextState::from(
            self.scheduler
                .next_state(state.map(MemoryState::from), days_elapsed)?,
        ))
    }
    // We have to arc proress (even though it already contains an arc) for ffi
    fn schedulerRecomputeParameters(
        &self,
        items: Vec<SchedulerItem>,
        progress: Option<Arc<Progress>>,
    ) -> Result<Vec<f32>, CoreError> {
        self.scheduler.compute_parameters(
            items.into_iter().map(From::from).collect::<Vec<_>>(),
            progress.map(|v| <Progress as Clone>::clone(&*v)),
        )
    }
    fn schedulerSetRetention(&self, value: f32) {
        self.scheduler.set_retention(value);
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
