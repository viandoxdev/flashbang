use std::{
    error::Error,
    io::{Cursor, Read},
    sync::Arc,
};

use fb_core::{
    cards::{CardState, SourceConfig},
    error::{AsCoreError, CoreError},
    typst::syntax::{FileId, Source, VirtualPath},
    world::{CardPage, FileSlot, WorldState},
};
use wasm_bindgen::prelude::*;
use zip::ZipArchive;

use crate::{card::Card, package_provider::ZippedPackageProvider};

mod card;
mod package_provider;

trait ToJsError<T> {
    fn to_js(self) -> Result<T, JsError>;
}

impl<T, E: Error> ToJsError<T> for Result<T, E> {
    fn to_js(self) -> Result<T, JsError> {
        match self {
            Ok(v) => Ok(v),
            Err(err) => Err(JsError::new(&err.to_string())),
        }
    }
}

#[wasm_bindgen]
pub struct Core {
    card: CardState,
    world: WorldState,
}

fn provide_repository_files(world: &WorldState, data: Arc<[u8]>) -> Result<(), CoreError> {
    let mut archive = ZipArchive::new(Cursor::new(data)).context(Some("Zip"))?;

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(file) => file,
            Err(err) => {
                log::error!("Can't access file {i} in archive: {err}");
                continue;
            }
        };

        let Some(path) = file.enclosed_name() else {
            log::warn!("File in archive contains malformed path.");
            continue;
        };

        let Ok(stripped) = path.strip_prefix("repo") else {
            continue;
        };

        let mut content = String::new();
        if let Err(err) = file.read_to_string(&mut content) {
            log::error!("Couldn't read file in repo archive at {stripped:?} : {err}");
            continue;
        }

        let id = FileId::new(None, VirtualPath::new(stripped));

        world.load_file(FileSlot::with_source(id, Source::new(id, content)));
    }

    Ok(())
}

#[wasm_bindgen]
impl Core {
    #[wasm_bindgen(constructor)]
    pub fn new(data: Vec<u8>) -> Result<Self, JsError> {
        let data: Arc<[u8]> = data.into();
        let package_provider = ZippedPackageProvider::new(data.clone()).to_js()?;
        let world = WorldState::new(package_provider);

        provide_repository_files(&world, data).to_js()?;

        // TODO: Load include files
        Ok(Self {
            world,
            card: CardState::new(),
        })
    }

    pub fn prepare_source(&self, cards: Vec<Card>, config: SourceConfig) -> Result<(), JsError> {
        self.world.prepare_source(&self.card, cards, config).to_js()
    }

    pub fn compile(&self) -> Result<Vec<CardPage>, JsError> {
        self.world.compile().to_js()
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("Error setting up console logger");
}
