use std::sync::Arc;

use nucleo::{
    Injector, Nucleo,
    pattern::{CaseMatching, Normalization},
};
use parking_lot::Mutex;

#[cfg_attr(feature = "uniffi", uniffi::export(with_foreign))]
pub trait FuzzyItem: Send + Sync + 'static {
    fn key(&self) -> String;
    fn data(&self) -> String;
}

type AnyFuzzy = Arc<dyn FuzzyItem>;

impl FuzzyItem for AnyFuzzy {
    fn key(&self) -> String {
        (**self).key()
    }
    fn data(&self) -> String {
        (**self).data()
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "uniffi", derive(uniffi::Enum))]
#[cfg_attr(feature = "wasm", wasm_bindgen::prelude::wasm_bindgen)]
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

pub struct FuzzyState<T = AnyFuzzy>
where
    T: FuzzyItem,
{
    /// Injector for fuzzy matcher
    pub injector: Arc<Mutex<Injector<T>>>,
    nucleo: Arc<Mutex<Nucleo<T>>>,
}

impl<T: FuzzyItem> FuzzyState<T> {
    pub fn new() -> Self {
        let nucleo = Nucleo::<T>::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 1);
        Self {
            injector: Arc::new(Mutex::new(nucleo.injector())),
            nucleo: Arc::new(Mutex::new(nucleo)),
        }
    }
}

impl<T: FuzzyItem> FuzzyState<T> {
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

    pub fn with_results<O>(
        &self,
        fun: impl FnOnce(&mut dyn ExactSizeIterator<Item = nucleo::Item<'_, T>>) -> O
    ) -> O {
        let n = self.nucleo.lock();
        let mut iterator = n.snapshot().matched_items(..);

        fun(&mut iterator)
    }

    pub fn add_item(&self, item: T) {
        let key = item.key();
        self.injector
            .lock()
            .push(item, |_, row| row[0] = key.into());
    }

    pub fn add_items(&self, items: Vec<T>) {
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
