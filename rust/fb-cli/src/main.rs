use std::{error::Error, path::Path};

use fb_core::{
    cards::{CardInfo, CardState},
    error::{AsCoreError, CoreError},
};
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
struct Card {
    pub id: String,
    pub name: String,
    pub locations: Vec<String>,
    pub header: Option<String>,
    pub question: String,
    pub answer: String,
}

impl From<CardInfo> for Card {
    fn from(value: CardInfo) -> Self {
        Self {
            id: value.id,
            name: value.name,
            locations: value.locations,
            header: value.header.map(|h| h.inner.clone()),
            question: value.question,
            answer: value.answer,
        }
    }
}

fn main() -> Result<(), CoreError> {
    pretty_env_logger::init();

    let args: Vec<String> = std::env::args().collect();

    let card = CardState::new();

    let search_path_str = &args[1];
    let output_file_str = &args[2];
    let output_directory_str = &args[3];

    let search_path = Path::new(search_path_str).canonicalize()?;

    let mut excluded_paths = args[4..]
        .iter()
        .filter_map(|exclude| Path::new(exclude).canonicalize().ok())
        .collect::<Vec<_>>();

    std::fs::create_dir_all(output_directory_str)?;
    let output_directory_path = Path::new(output_directory_str).canonicalize()?;
    let output_file_path = Path::new(output_file_str);

    excluded_paths.push(output_directory_path.clone());

    let mut errors = Vec::<Box<dyn Error>>::new();
    let mut cards = Vec::new();

    for (id, entry) in WalkDir::new(&search_path)
        .into_iter()
        .filter_entry(|entry| {
            let Ok(can) = entry.path().canonicalize() else {
                return false;
            };

            !excluded_paths.contains(&can)
        })
        .enumerate()
    {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                errors.push(Box::new(err));
                continue;
            }
        };

        if entry.path().extension().and_then(|s| s.to_str()) != Some("typ") {
            continue;
        }

        let content = match std::fs::read_to_string(entry.path()) {
            Ok(content) => content,
            Err(err) => {
                errors.push(Box::new(err));
                continue;
            }
        };

        if content.starts_with("//![FLASHBANG IGNORE]") {
            continue;
        }

        if content.starts_with("//![FLASHBANG INCLUDE]") {
            // This shouldn't error out as if we managed to read the file already
            let absolute_path = entry.path().canonicalize()?;
            let relative_path = absolute_path
                .strip_prefix(&search_path)
                .context(Some("Strip Prefix"))?;
            let new_path = output_directory_path.join(relative_path);

            std::fs::create_dir_all(new_path.parent().expect("This shouldn't be at root"))?;
            std::fs::write(new_path, content)?;
            continue;
        }

        log::debug!("Parsing {:?}", entry.path());

        match card.parse(id as u64, &content) {
            Ok(new_cards) => cards.extend(new_cards.into_iter().map(Card::from)),
            Err(err) => errors.push(Box::new(err)),
        }
    }

    let json_string = serde_json::to_string(&cards).context(Some("Serialization"))?;

    std::fs::write(output_file_path, json_string)?;

    Ok(())
}
