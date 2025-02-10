use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    cards::Rating,
    storage::{Serializable, TypeList},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CardView {
    pub timestamp: u64,
    pub rating: Rating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardInfo {
    pub score: u8,
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
