use fb_core::fuzzy::FuzzyItem;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone)]
#[wasm_bindgen]
pub struct Fuzzy {
    key: String,
    data: String,
}

#[wasm_bindgen]
impl Fuzzy {
    #[wasm_bindgen(constructor)]
    pub fn new(key: String, data: String) -> Self {
        Self {
            key, data
        }
    }

    pub fn key(&self) -> String {
        self.key.clone()
    }

    pub fn data(&self) -> String {
        self.data.clone()
    }
}

impl FuzzyItem for Fuzzy {
    fn key(&self) -> String {
        self.key.clone()
    }
    fn data(&self) -> String {
        self.data.clone()
    }
}
