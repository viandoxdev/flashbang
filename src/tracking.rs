use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use dioxus::logger::tracing::warn;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MemoryState {
    pub stability: f32,
    pub difficulty: f32,
}

impl From<fsrs::MemoryState> for MemoryState {
    fn from(value: fsrs::MemoryState) -> Self {
        Self {
            stability: value.stability,
            difficulty: value.difficulty,
        }
    }
}

impl From<MemoryState> for fsrs::MemoryState {
    fn from(value: MemoryState) -> Self {
        Self {
            stability: value.stability,
            difficulty: value.difficulty,
        }
    }
}

use crate::{
    algorithm,
    cards::{CardHandle, Rating},
    deck::store,
    storage::{Serializable, Storable, TypeList},
};

static TRACKING: OnceLock<Arc<Mutex<Storable<TrackedData>>>> = OnceLock::new();

pub fn tracking() -> &'static Arc<Mutex<Storable<TrackedData>>> {
    TRACKING.get().unwrap()
}

pub fn init_tracking() {
    TRACKING.get_or_init(|| Arc::new(Mutex::new(Storable::new("tracking", TrackedData::default))));
    let tracking = tracking().lock();
    algorithm::init_fsrs(&tracking.fsrs_params);
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CardReview {
    pub timestamp: i64,
    pub rating: Rating,
}

impl CardReview {
    fn to_fsrs_review(self, prev: i64) -> fsrs::FSRSReview {
        fsrs::FSRSReview {
            rating: self.rating.score() + 1,
            // Should I convert to Date to do the math or is this robust enough ?
            delta_t: ((self.timestamp - prev) / (24 * 60 * 60)) as u32,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CardInfo {
    pub reviews: Vec<CardReview>,
    pub memory_state: Option<MemoryState>,
    pub due: Option<i64>,
}

impl From<&CardInfo> for fsrs::FSRSItem {
    fn from(value: &CardInfo) -> Self {
        let first_review = value
            .reviews
            .first()
            .map(|rev| rev.timestamp)
            .unwrap_or_default();
        let cap = value.reviews.len();
        Self {
            reviews: value
                .reviews
                .iter()
                .fold(
                    (Vec::with_capacity(cap), first_review),
                    |(mut revs, prev), rev| {
                        revs.push(rev.to_fsrs_review(prev));
                        (revs, rev.timestamp)
                    },
                )
                .0,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Session {
    pub timestamp: i64,
    pub size: u16,
    pub score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedData {
    pub cards_info: HashMap<String, CardInfo>,
    pub sessions: Vec<Session>,
    pub fsrs_params: Vec<f32>,
    pub repo_sha: String,
}

impl TrackedData {
    pub fn update_params(&mut self, params: Vec<f32>) {
        algorithm::set_params(&params);
        self.fsrs_params = params;
    }
    pub fn add_session(
        &mut self,
        timestamp: i64,
        retention: f32,
        answers: impl IntoIterator<Item = (CardHandle, Rating)>,
    ) {
        let mut size = 0u16;
        let mut score = 0u32;
        let store = store().lock();
        for (card, rating) in answers {
            let review = CardReview { timestamp, rating };
            let id = &store.cards[card].id;
            let card = self.cards_info.entry(id.to_owned()).or_default();

            if let Err(e) = algorithm::update_card(card, review, retention) {
                warn!("Error while updating card: {e} ({e:?})");
            }

            size += 1;
            score += rating.score();
        }
        let session = Session {
            timestamp,
            size,
            score: (score * 100 / 4 / size as u32) as u8,
        };

        self.sessions.push(session);
    }
}

impl Default for TrackedData {
    fn default() -> Self {
        Self {
            cards_info: HashMap::new(),
            sessions: Vec::new(),
            fsrs_params: fsrs::DEFAULT_PARAMETERS.to_vec(),
            repo_sha: "".to_owned(),
        }
    }
}

impl Serializable for TrackedData {
    type Fallback = TypeList<()>;
}
