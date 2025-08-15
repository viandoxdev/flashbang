use std::{collections::HashMap, sync::{atomic::AtomicBool, Arc}};

use parking_lot::Mutex;

use crate::{arc_struct, cards::{Card, Rating}};

arc_struct!{
    pub struct Study {
        inner StudyInner {
            pub id: u64,
            timestamp: u64,
            selection: Vec<Card>,
        }

        state StudyState {
            name: String,
            ratings: HashMap<String, Rating>,
            finished: bool,
        }
    }
}

impl Study {
    pub fn new(id: u64, name: String, timestamp: u64, selection: Vec<Card>) -> Self {
        Self(Arc::new(StudyInner {
            id,
            timestamp,
            selection,
            state: Mutex::new(StudyState {
                name: name,
                ratings: HashMap::new(),
                finished: false,
            })
        }))
    }

    pub fn rename(&self, name: String) {
        self.state.lock().name = name;
    }

    pub fn update(&self, rating: Rating, card_id: String) {
        self.state.lock().ratings.insert(card_id, rating);
    }

    pub fn finalize(&self) {
        self.state.lock().finished = true
    }
}
