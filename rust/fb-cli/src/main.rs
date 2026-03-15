use std::{error::Error, path::PathBuf, time::Instant};

use clap::Parser;
use colored::Colorize;
use fb_core::{
    cards::{CardInfo, CardState},
    error::{AsCoreError, CoreError},
};
use serde::Serialize;
use walkdir::WalkDir;

#[derive(Parser)]
#[command(author, version, about = "Flashbang CLI - A tool to parse flashcards from Typst files", long_about = None)]
struct Cli {
    /// Directory to search for .typ files
    #[arg(short, long)]
    search_path: PathBuf,

    /// Output JSON file for cards
    #[arg(short, long)]
    output_file: PathBuf,

    /// Output directory for included assets
    #[arg(short = 'd', long)]
    output_dir: PathBuf,

    /// Paths to exclude from searching
    #[arg(short, long, action = clap::ArgAction::Append)]
    exclude: Vec<PathBuf>,
}

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
    if std::env::var("RUST_LOG").is_err() {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    pretty_env_logger::init();

    let cli = Cli::parse();
    let start_time = Instant::now();

    let search_path = cli
        .search_path
        .canonicalize()
        .context(Some("Search Path"))?;
    let output_dir = cli.output_dir.clone();
    std::fs::create_dir_all(&output_dir).context(Some("Output Dir Creation"))?;
    let output_dir_canonical = output_dir.canonicalize().context(Some("Output Dir"))?;

    let mut excluded_paths = cli
        .exclude
        .iter()
        .filter_map(|path| path.canonicalize().ok())
        .collect::<Vec<_>>();

    excluded_paths.push(output_dir_canonical.clone());

    // Print summary of arguments
    println!("{}", "Flashbang CLI Config".bold());
    println!("{:>12}: {}", "Search Path", search_path.display().to_string().cyan());
    println!("{:>12}: {}", "Output JSON", cli.output_file.display().to_string().cyan());
    println!("{:>12}: {}", "Asset Dir", output_dir_canonical.display().to_string().cyan());
    if !cli.exclude.is_empty() {
        println!("{:>12}:", "Excluded");
        for path in &cli.exclude {
            println!("{:>15} {}", "-", path.display().to_string().yellow());
        }
    }
    println!();

    let card_state = CardState::new();
    let mut errors = Vec::<Box<dyn Error>>::new();
    let mut cards = Vec::new();
    let mut asset_count = 0;
    let mut excluded_count = 0;
    let mut card_file_count = 0;

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

        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("typ") {
            continue;
        }

        let relative_path = path
            .strip_prefix(&search_path)
            .unwrap_or(path)
            .display()
            .to_string();

        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(err) => {
                errors.push(Box::new(err));
                continue;
            }
        };

        if content.starts_with("//![FLASHBANG IGNORE]") {
            println!("{:>10} {}", "EXCLUDED".yellow().bold(), relative_path);
            excluded_count += 1;
            continue;
        }

        if content.starts_with("//![FLASHBANG INCLUDE]") {
            let absolute_path = entry.path().canonicalize()?;
            let rel_to_search = absolute_path
                .strip_prefix(&search_path)
                .context(Some("Strip Prefix"))?;
            let new_path = output_dir_canonical.join(rel_to_search);

            if let Some(parent) = new_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(new_path, content)?;
            println!("{:>10} {}", "ASSET".blue().bold(), relative_path);
            asset_count += 1;
            continue;
        }

        log::debug!("Parsing {:?}", entry.path());

        match card_state.parse(id as u64, &content) {
            Ok(new_cards) => {
                if !new_cards.is_empty() {
                    println!(
                        "{:>10} {} ({} cards)",
                        "CARDS".green().bold(),
                        relative_path,
                        new_cards.len()
                    );
                    cards.extend(new_cards.into_iter().map(Card::from));
                    card_file_count += 1;
                } else {
                    // Maybe it's a helper file or empty?
                    println!("{:>10} {}", "EMPTY".truecolor(128, 128, 128), relative_path);
                }
            }
            Err(err) => {
                println!("{:>10} {}", "ERROR".red().bold(), relative_path);
                errors.push(Box::new(err));
            }
        }
    }

    if !errors.is_empty() {
        println!("\n{}", "Errors encountered:".red().bold());
        for error in &errors {
            println!("  {} {}", "•".red(), error);
        }
    }

    let json_string = serde_json::to_string(&cards).context(Some("Serialization"))?;
    std::fs::write(&cli.output_file, json_string).context(Some("Writing JSON"))?;

    let duration = start_time.elapsed();

    println!("\n{}", "Summary".bold());
    println!("{:>15} {}", "Total Cards:", cards.len().to_string().green().bold());
    println!("{:>15} {}", "Files w/ Cards:", card_file_count);
    println!("{:>15} {}", "Assets Copied:", asset_count.to_string().blue());
    println!("{:>15} {}", "Files Ignored:", excluded_count.to_string().yellow());
    println!("{:>15} {}", "Errors:", errors.len().to_string().red());
    println!("{:>15} {:.2?}", "Time Taken:", duration);

    Ok(())
}
