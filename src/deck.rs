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

static STORE: OnceLock<Arc<Mutex<CardStore>>> = OnceLock::new();

pub fn store() -> &'static Arc<Mutex<CardStore>> {
    STORE.get().unwrap()
}

pub fn init_store(store: impl FnOnce() -> CardStore) {
    STORE.get_or_init(|| Arc::new(Mutex::new(store())));
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

#[component]
pub fn Deck(width: u32, cards: ReadOnlySignal<Vec<CardHandle>>) -> Element {
    if cards.is_empty() {
        Err(CapturedError::from_display("No cards in deck"))?
    }

    let settings: Signal<Storable<Settings>> = use_context();

    let config = SourceConfig {
        page_width: (width.saturating_sub(50)) * 3 / 4,
        text_size: settings.read().font_size,
    };

    let pages = if width > 0 {
        let store = store().lock();
        let content = store.build_source(cards().iter().copied(), config)?;
        typst::compile(&TypstWrapper::new("./", &content))
            .output
            .map(|doc| doc.pages)
            .map_err(|err| CapturedError::from_display(format!("{err:?}")))?
    } else {
        vec![]
    };

    let mut answering = use_signal(|| true);
    let mut results = use_signal(SecondaryMap::<CardHandle, Rating>::new);
    let mut index = use_signal(|| 0);

    rsx! {
        div {
            class: "deck",
            if index() < cards.len() {
                div {
                    class: "card",
                    dangerous_inner_html: "{
                        pages.get(index * 2 + 1 + (!answering()) as usize)
                            .map(|page| typst_svg::svg(page))
                            .unwrap_or_default()
                    }"
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
