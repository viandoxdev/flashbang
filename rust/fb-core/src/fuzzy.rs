use std::sync::Arc;

use itertools::Itertools;
use nucleo::{pattern::{CaseMatching, Normalization}, Injector, Nucleo};
use parking_lot::Mutex;

#[cfg_attr(feature = "uniffi", uniffi::export(with_foreign))]
pub trait FuzzyItem: Send + Sync {
    fn key(&self) -> String;
    fn data(&self) -> String;
}

type AnyFuzzy = Arc<dyn FuzzyItem>;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
pub enum FuzzyStatus {
    Stale,
    Updated,
    Finished,
}

impl From<nucleo::Status> for FuzzyStatus {
    fn from(value: nucleo::Status) -> Self {
        if !value.running {
            Self::Finished
        } else if value.changed {
            Self::Updated
        } else {
            Self::Stale
        }
    }
}

pub struct FuzzyState {
    /// Injector for fuzzy matcher
    pub injector: Arc<Mutex<Injector<AnyFuzzy>>>,
    nucleo: Arc<Mutex<Nucleo<AnyFuzzy>>>,
}

impl FuzzyState {
    pub fn new() -> Self {
        let nucleo = Nucleo::<AnyFuzzy>::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 1);
        Self {
            injector: Arc::new(Mutex::new(nucleo.injector())),
            nucleo: Arc::new(Mutex::new(nucleo)),
        }
    }
}

impl FuzzyState {
    pub fn init(&self, pattern: &str) {
        self.nucleo.lock().pattern.reparse(
            0,
            pattern,
            CaseMatching::Ignore,
            Normalization::Smart,
            false,
        );
    }

    pub fn tick(&self) -> FuzzyStatus {
        self.nucleo.lock().tick(500).into()
    }

    pub fn results(&self) -> Vec<AnyFuzzy> {
        self.nucleo
            .lock()
            .snapshot()
            .matched_items(..)
            .map(|item| item.data.clone())
            .collect_vec()
    }

    pub fn add_item(&self, item: AnyFuzzy) {
        let key = item.key();
        self.injector.lock().push(item, |_, row| row[0] = key.into());
    }

    pub fn add_items(&self, items: Vec<AnyFuzzy>) {
        let injector = self.injector.lock();
        for item in items {
            let key = item.key();
            injector.push(item, |_, row| row[0] = key.into());
        }
    }

    pub fn reset(&self) {
        self.nucleo.lock().restart(true);
        *self.injector.lock() = self.nucleo.lock().injector();
    }
}
