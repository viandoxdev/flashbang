use std::{error::Error, sync::Arc};

use fb_core::{cards::{CardState, SourceConfig}, fuzzy::{FuzzyState, FuzzyStatus}, world::{CardPage, WorldState}};
use wasm_bindgen::prelude::*;

use crate::{card::Card, fuzzy::Fuzzy, package_provider::ZippedPackageProvider};

mod package_provider;
mod card;
mod fuzzy;

trait ToJsError<T> {
    fn to_js(self) -> Result<T, JsError>;
}

impl<T, E: Error> ToJsError<T> for Result<T, E> {
    fn to_js(self) -> Result<T, JsError> {
        match self {
            Ok(v) => Ok(v),
            Err(err) => Err(JsError::new(&err.to_string()))
        }
    }
}

#[wasm_bindgen]
pub struct Core {
    card: CardState,
    world: WorldState,
    fuzzy: FuzzyState<Fuzzy>,
}

#[wasm_bindgen]
impl Core {
    #[wasm_bindgen(constructor)]
    pub fn new(data: Vec<u8>) -> Result<Self, JsError> {
        let data: Arc<[u8]> = data.into();
        let package_provider = ZippedPackageProvider::new(data.clone()).to_js()?;

        // TODO: Load include files
        Ok(Self {
            world: WorldState::new(package_provider),
            card: CardState::new(),
            fuzzy: FuzzyState::new(),
        })
    }

    pub fn prepare_source(&self, cards: Vec<Card>, config: SourceConfig) -> Result<(), JsError> {
        self.world.prepare_source(&self.card, cards, config).to_js()
    }

    pub fn compile(&self) -> Result<Vec<CardPage>, JsError> {
        self.world.compile().to_js()
    }
    pub fn fuzzy_init(&self, pattern: String) {
        self.fuzzy.init(&pattern);
    }
    pub fn fuzzy_tick(&self) -> FuzzyStatus {
        self.fuzzy.tick()
    }
    pub fn fuzzy_results(&self) -> Vec<Fuzzy> {
        self.fuzzy.with_results(|iter| iter.map(|i| i.data.clone()).collect())
    }
    pub fn fuzzy_add_items(&self, items: Vec<Fuzzy>) {
        self.fuzzy.add_items(items);
    }
    pub fn fuzzy_reset(&self) {
        self.fuzzy.reset();
    }
}
