use std::sync::{Arc, OnceLock};

use crate::cards::{CardHandle, CardStore, Rating, SourceConfig, RATINGS};
use crate::settings::Settings;
use crate::storage::Storable;
use crate::tracking::tracking;
use crate::typst_wrap::TypstWrapper;
use crate::AppState;
use dioxus::{prelude::*, CapturedError};
use itertools::Itertools;
use parking_lot::Mutex;
use slotmap::SecondaryMap;
use time::OffsetDateTime;
use typst::layout::Page;

static STORE: OnceLock<Arc<Mutex<CardStore>>> = OnceLock::new();

pub fn store() -> &'static Arc<Mutex<CardStore>> {
    STORE.get().unwrap()
}

pub fn init_store(store: CardStore) {
    STORE.get_or_init(|| Arc::new(Mutex::new(store)));
}

#[component]
pub fn Results(results: SecondaryMap<CardHandle, Rating>) -> Element {
    let settings: Signal<Storable<Settings>> = use_context();

    let results = Arc::new(results);
    let by_ratings = results.values().copied().counts();
    let score = results.values().map(|r| r.score()).sum::<u32>() * 100 / (3 * results.len() as u32);
    rsx! {
        div {
            class: "results",
            div {
                class: "label label-summary",
                "Summary"
            }
            div {
                class: "summary",
                for rating in RATINGS {
                    div {
                        class: "label rate-{rating.str().to_lowercase()}",
                        "{rating}"
                    }
                    div {
                        class: "bar rate-{rating.str().to_lowercase()}",
                        width: "{
                            by_ratings.get(&rating).copied().unwrap_or_default() as f32
                            / results.len() as f32 * 100.0
                        }%"
                    }
                    div {
                        class: "count",
                        "{by_ratings.get(&rating).copied().unwrap_or_default()}"
                    }
                }
                div {
                    class: "total",
                    "Out of {results.len()} cards"
                }
            }
            div {
                class: "score",
                "{score}%"
            }
            div {
                class: "label label-score",
                "Your score"
            }
            button {
                class: "finish",
                onclick: move |_| {
                    let results = results.clone();
                    spawn(async move {
                        let timestamp = OffsetDateTime::now_utc().unix_timestamp();
                        let retention = settings.read().retention;
                        let mut tracking = tracking().lock();
                        tracking.add_session(timestamp, retention, results.iter().map(|(k, &v)| (k, v)));
                        tracking.save();

                        let mut state = use_context::<Signal<AppState>>();
                        state.set(AppState::Home);
                    });
                },
                "Finish !"
            }
        }
    }
}

// Hacky, ugly, but Page doesn't impl PartialEqual
#[derive(Clone)]
struct CompiledPages {
    output: Vec<Page>,
    width: u32,
    font_size: u32,
    cards: Vec<CardHandle>,
}

impl CompiledPages {
    fn new(width: u32, font_size: u32, cards: Vec<CardHandle>) -> Result<Self, CapturedError> {
        let config = SourceConfig {
            page_width: (width.saturating_sub(50)) * 3 / 4,
            text_size: font_size,
        };
        let output = if width > 0 {
            let store = store().lock();
            let content = store.build_source(cards.iter().copied(), config)?;
            typst::compile(&TypstWrapper::new("./", &content))
                .output
                .map(|doc| doc.pages)
                .map_err(|err| CapturedError::from_display(format!("{err:?}")))
        } else {
            Ok(vec![])
        }?;

        Ok(Self {
            cards,
            font_size,
            output,
            width,
        })
    }

    fn svg(&self, index: usize, answering: bool) -> String {
        self.output
            .get(index * 2 + 1 + (!answering) as usize)
            .map(typst_svg::svg)
            .unwrap_or_default()
    }
}

impl PartialEq for CompiledPages {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && self.cards == other.cards && self.font_size == other.font_size
    }
}

#[component]
pub fn Deck(width: u32, cards: ReadOnlySignal<Vec<CardHandle>>) -> Element {
    if cards.is_empty() {
        Err(CapturedError::from_display("No cards in deck"))?
    }

    let settings: Signal<Storable<Settings>> = use_context();

    let pages = use_memo(use_reactive!(|(width,)| CompiledPages::new(
        width,
        settings.read().font_size,
        cards()
    )));

    if let Some(err) = pages.peek().as_ref().err() {
        Err(err.clone())?
    }

    let mut answering = use_signal(|| true);
    let mut results = use_signal(SecondaryMap::<CardHandle, Rating>::new);
    let mut index = use_signal(|| 0);

    let svg = use_memo(move || {
        let index = index();
        let answering = answering();
        pages
            .read()
            .as_ref()
            .ok()
            .map(move |c| c.svg(index, answering))
            .unwrap_or_default()
    });

    rsx! {
        div {
            class: "deck",
            if index() < cards.len() {
                div {
                    class: "card",
                    dangerous_inner_html: "{svg}"
                }
                div {
                    class: "controls",
                    if answering() {
                        button {
                            class: "reveal",
                            onclick: move |_| answering.set(false),
                            "Reveal"
                        }
                    } else {
                        for rating in RATINGS {
                            button {
                                class: "rate-{rating.str().to_lowercase()}",
                                onclick: move |_| {
                                    results.write().insert(cards()[index()], rating);
                                    answering.set(true);
                                    *index.write() += 1;
                                },
                                "{rating}"
                            }
                        }
                    }
                }
            } else {
                Results {
                    results: results()
                }
            }
        }
    }
}
