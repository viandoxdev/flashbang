use std::{
    collections::HashMap,
    sync::{Arc, OnceLock},
};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::{
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
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CardView {
    pub timestamp: u64,
    pub rating: Rating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardInfo {
    pub score: i8,
    pub views: Vec<CardView>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Session {
    pub timestamp: u64,
    pub size: u16,
    pub score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedData {
    pub cards_info: HashMap<String, CardInfo>,
    pub sessions: Vec<Session>,
}

impl TrackedData {
    pub fn add_session(
        &mut self,
        timestamp: u64,
        answers: impl IntoIterator<Item = (CardHandle, Rating)>,
    ) {
        let mut size = 0u16;
        let mut score = 0u32;
        let store = store().lock();
        for (card, rating) in answers {
            let view = CardView { timestamp, rating };
            let id = &store.cards[card].id;
            let card = self.cards_info.get_mut(id).unwrap();

            card.views.push(view);
            card.score = (card.score + rating.points()).clamp(0, 100);
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
        }
    }
}

impl Serializable for TrackedData {
    type Fallback = TypeList<()>;
}
