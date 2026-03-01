use std::sync::Arc;

use itertools::Itertools;
use nucleo::{
    Injector, Nucleo,
    pattern::{CaseMatching, Normalization},
};
use parking_lot::Mutex;

use crate::Core;

#[uniffi::export(with_foreign)]
pub trait FuzzyItem: Send + Sync {
    fn key(&self) -> String;
    fn data(&self) -> String;
}

type AnyFuzzy = Arc<dyn FuzzyItem>;

#[derive(Debug, Clone, Copy, uniffi::Enum)]
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

pub trait FuzzyCore {
    fn init(&self, pattern: &str);
    fn tick(&self) -> FuzzyStatus;
    fn results(&self) -> Vec<AnyFuzzy>;
    fn add_item(&self, item: AnyFuzzy);
    fn add_items(&self, items: Vec<AnyFuzzy>);
    fn reset(&self);
}

impl FuzzyCore for Core {
    fn init(&self, pattern: &str) {
        self.fuzzy.nucleo.lock().pattern.reparse(
            0,
            pattern,
            CaseMatching::Ignore,
            Normalization::Smart,
            false,
        );
    }

    fn tick(&self) -> FuzzyStatus {
        self.fuzzy.nucleo.lock().tick(500).into()
    }

    fn results(&self) -> Vec<AnyFuzzy> {
        self.fuzzy
            .nucleo
            .lock()
            .snapshot()
            .matched_items(..)
            .map(|item| item.data.clone())
            .collect_vec()
    }

    fn add_item(&self, item: AnyFuzzy) {
        let key = item.key();
        self.fuzzy
            .injector
            .lock()
            .push(item, |_, row| row[0] = key.into());
    }

    fn add_items(&self, items: Vec<AnyFuzzy>) {
        let injector = self.fuzzy.injector.lock();
        for item in items {
            let key = item.key();
            injector.push(item, |_, row| row[0] = key.into());
        }
    }

    fn add_items(&self, items: Vec<AnyFuzzy>) {
        let injector = self.fuzzy.injector.lock();
        for item in items {
            let key = item.key();
            injector.push(item, |_, row| row[0] = key.into());
        }
    }

    fn reset(&self) {
        self.fuzzy.nucleo.lock().restart(true);
        *self.fuzzy.injector.lock() = self.fuzzy.nucleo.lock().injector();
    }
}
