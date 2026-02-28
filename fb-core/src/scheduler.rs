use std::sync::{Arc, atomic::AtomicU32};

use fsrs::{CombinedProgressState, FSRS, FSRSItem, FSRSReview, ItemState, MemoryState, NextStates};
use itertools::Itertools;
use parking_lot::Mutex;

use crate::{Core, error::CoreError};

#[derive(Debug, Clone, uniffi::Object)]
pub struct Progress {
    inner: Arc<std::sync::Mutex<CombinedProgressState>>,
}

impl Progress {
    fn as_raw(&self) -> Arc<std::sync::Mutex<CombinedProgressState>> {
        self.inner.clone()
    }
}

#[uniffi::export]
impl Progress {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            inner: CombinedProgressState::new_shared(),
        }
    }

    pub fn read(&self) -> f32 {
        if let Ok(prog) = self.inner.lock() {
            prog.current() as f32 / prog.total() as f32
        } else {
            0.0
        }
    }
}

pub struct SchedulerState {
    fsrs: Arc<Mutex<FSRS>>,
    retention: AtomicU32,
}

impl SchedulerState {
    /// Nonsense default value, will be overridden by another (petentially default) value anyways
    const DEFAULT_RENTENTION: f32 = 0.5;
    pub fn new() -> Result<Self, CoreError> {
        Ok(Self {
            fsrs: Arc::new(Mutex::new(FSRS::new(Some(&[]))?)),
            retention: AtomicU32::new(Self::DEFAULT_RENTENTION.to_bits()),
        })
    }
}

pub trait SchedulerCore {
    fn set_parameters(&self, parameters: &[f32]) -> Result<(), CoreError>;
    fn set_retention(&self, value: f32);
    fn next_state(
        &self,
        state: Option<MemoryState>,
        days_elapsed: u32,
    ) -> Result<NextStates, CoreError>;
    fn compute_parameters(
        &self,
        items: Vec<FSRSItem>,
        progress: Option<Progress>,
    ) -> Result<Vec<f32>, CoreError>;
}

impl SchedulerCore for Core {
    fn set_retention(&self, value: f32) {
        self.scheduler.set_retention(value);
    }
    fn set_parameters(&self, parameters: &[f32]) -> Result<(), CoreError> {
        *self.scheduler.fsrs.lock() = FSRS::new(Some(parameters))?;
        Ok(())
    }
    fn next_state(
        &self,
        state: Option<MemoryState>,
        days_elapsed: u32,
    ) -> Result<NextStates, CoreError> {
        Ok(self.scheduler.fsrs.lock().next_states(
            state,
            self.scheduler.get_retention(),
            days_elapsed,
        )?)
    }
    fn compute_parameters(
        &self,
        items: Vec<FSRSItem>,
        progress: Option<Progress>,
    ) -> Result<Vec<f32>, CoreError> {
        Ok(self
            .scheduler
            .fsrs
            .lock()
            .compute_parameters(fsrs::ComputeParametersInput {
                train_set: items,
                progress: progress.map(|prog| prog.as_raw()),
                enable_short_term: false,
                num_relearning_steps: None,
            })?)
    }
}

impl SchedulerState {
    fn set_retention(&self, value: f32) {
        self.retention
            .store(value.to_bits(), std::sync::atomic::Ordering::SeqCst);
    }
    fn get_retention(&self) -> f32 {
        f32::from_bits(self.retention.load(std::sync::atomic::Ordering::SeqCst))
    }
}

// Uniffi wrapper types

#[derive(uniffi::Record)]
pub struct SchedulerMemoryState {
    stability: f32,
    difficulty: f32,
}

#[derive(uniffi::Record)]
pub struct SchedulerReview {
    rating: u32,
    delta_t: u32,
}

#[derive(uniffi::Record)]
pub struct SchedulerItemState {
    state: SchedulerMemoryState,
    delay: f32,
}

#[derive(uniffi::Record)]
pub struct SchedulerNextState {
    again: SchedulerItemState,
    hard: SchedulerItemState,
    good: SchedulerItemState,
    easy: SchedulerItemState,
}

pub struct SchedulerItem(Vec<SchedulerReview>);

uniffi::custom_newtype!(SchedulerItem, Vec<SchedulerReview>);

impl From<SchedulerMemoryState> for MemoryState {
    fn from(value: SchedulerMemoryState) -> Self {
        Self {
            stability: value.stability,
            difficulty: value.difficulty,
        }
    }
}

impl From<MemoryState> for SchedulerMemoryState {
    fn from(value: MemoryState) -> Self {
        Self {
            stability: value.stability,
            difficulty: value.difficulty,
        }
    }
}

impl From<SchedulerReview> for FSRSReview {
    fn from(value: SchedulerReview) -> Self {
        Self {
            rating: value.rating,
            delta_t: value.delta_t,
        }
    }
}

impl From<SchedulerItem> for FSRSItem {
    fn from(value: SchedulerItem) -> Self {
        Self {
            reviews: value.0.into_iter().map(From::from).collect_vec(),
        }
    }
}

impl From<ItemState> for SchedulerItemState {
    fn from(value: ItemState) -> Self {
        Self {
            state: From::from(value.memory),
            delay: value.interval,
        }
    }
}

impl From<NextStates> for SchedulerNextState {
    fn from(value: NextStates) -> Self {
        Self {
            hard: From::from(value.hard),
            again: From::from(value.again),
            good: From::from(value.good),
            easy: From::from(value.easy),
        }
    }
}
