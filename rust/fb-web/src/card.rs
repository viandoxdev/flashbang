use fb_core::cards::CardSource;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Card {
    header: Option<String>,
    id: String,
    name: String,
    question: String,
    answer: String,
    locations: Vec<String>,
}

#[wasm_bindgen]
impl Card {
    #[wasm_bindgen(constructor)]
    pub fn new(
        header: Option<String>,
        id: String,
        name: String,
        question: String,
        answer: String,
        locations: Vec<String>,
    ) -> Self {
        Self {
            id,
            question,
            answer,
            locations,
            header,
            name,
        }
    }
}

impl CardSource for Card {
    fn header_content(&self) -> Option<String> {
        self.header.clone()
    }
    fn id(&self) -> String {
        self.id.clone()
    }
    fn name(&self) -> String {
        self.name.clone()
    }
    fn question(&self) -> String {
        self.question.clone()
    }
    fn answer(&self) -> String {
        self.answer.clone()
    }
    fn locations(&self) -> Vec<String> {
        self.locations.clone()
    }
}
