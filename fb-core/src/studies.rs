use std::{
    collections::HashMap,
    sync::{Arc, atomic::AtomicU64},
    time::SystemTime,
};

use log::error;
use parking_lot::Mutex;

use crate::{
    arc_struct,
    cards::{Card, Rating},
};

arc_struct! {
    pub struct Study {
        inner StudyInner {
            pub id: u64,
            timestamp: u64,
            selection: Vec<Card>,
        }

        #[derive(uniffi::Record)]
        state StudyState {
            name: String,
            ratings: HashMap<String, Rating>,
            finished: bool,
        }
    }
}

impl Study {
    pub fn new(id: u64, timestamp: u64, selection: Vec<Card>, state: StudyState) -> Self {
        Self(Arc::new(StudyInner {
            id,
            timestamp,
            selection,
            state: Mutex::new(state),
        }))
    }
}

pub struct StudyStore {
    studies: HashMap<u64, Study>,
    last_id: AtomicU64,
}

impl Default for StudyStore {
    fn default() -> Self {
        Self {
            studies: HashMap::new(),
            last_id: AtomicU64::new(0),
        }
    }
}

impl StudyStore {
    pub fn last_id(&self) -> Option<u64> {
        let id = self.last_id.load(std::sync::atomic::Ordering::SeqCst);
        (id != 0).then_some(id)
    }

    pub fn set_last_id(&self, value: u64) {
        self.last_id.store(value, std::sync::atomic::Ordering::SeqCst)
    }

    pub fn studies(&self) -> impl Iterator<Item = Study> {
        self.studies.values().cloned()
    }

    pub fn delete_study(&mut self, id: u64) -> Option<Study> {
        self.studies.remove(&id)
    }

    pub fn load_study(&mut self, id: u64, timestamp: u64, selection: Vec<Card>, state: StudyState) -> Study {
        if let Some(study) = self.studies.get(&id) {
            // These shouldn't be mutable
            if study.timestamp != timestamp {
                error!("Mismatched study timestamp ({id})");
            }
            if study.selection != selection {
                error!("Mismatched study selection ({id})");
            }

            *study.state.lock() = state;

            study.clone()
        } else {
            let study = Study::new(id, timestamp, selection, state);

            self.studies.insert(id, study.clone());

            study
        }
    }

    pub fn new_study(&mut self, name: String, selection: Vec<Card>) -> Study {
        let id = self
            .last_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        if id == 0 {
            panic!("New study created before id counter was set");
        }

        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|x| x.as_secs())
            .unwrap_or_default();
        self.load_study(id, timestamp, selection, StudyState {
            name,
            ratings: HashMap::new(),
            finished: false,
        })
    }
}

#[uniffi::export]
impl StudyInner {
    #[uniffi::method(name = "getId")]
    fn _get_id(&self) -> u64 {
        self.id
    }
    #[uniffi::method(name = "getTimestamp")]
    fn _get_timestamp(&self) -> u64 {
        self.timestamp
    }
    #[uniffi::method(name = "getSelection")]
    fn _get_selection(&self) -> Vec<Card> {
        self.selection.clone()
    }
    #[uniffi::method(name = "rename")]
    fn _rename(&self, name: String) {
        self.state.lock().name = name;
    }
    #[uniffi::method(name = "update")]
    fn _update(&self, rating: Rating, card_id: String) {
        self.state.lock().ratings.insert(card_id, rating);
    }
    #[uniffi::method(name = "finalize")]
    fn _finalize(&self) {
        self.state.lock().finished = true
    }
}
