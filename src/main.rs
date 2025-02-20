#![allow(dead_code)]

use cards::{CardHandle, CardStore, LoadError};
use deck::{init_store, Deck};
use dioxus::{logger::tracing::Level, prelude::*, CapturedError};
use dioxus_free_icons::{
    icons::{
        bs_icons as bs,
        fa_solid_icons::{self as fa, FaAngleLeft},
        hi_solid_icons as hi,
        ld_icons::LdLoaderCircle,
        md_alert_icons::MdError,
    },
    Icon,
};

use itertools::Itertools;
use popup::ToastDisplay;
use selection::Selection;
use settings::{Settings, SettingsComponent};
use stats::Stats;
use storage::Storable;
use tracking::init_tracking;

mod algorithm;
mod cards;
mod deck;
mod github;
mod popup;
mod selection;
mod settings;
mod stats;
mod storage;
mod tracking;
mod typst_wrap;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const FONT_AGBALUMO: Asset = asset!("/assets/Agbalumo.woff2");
const FONT_LEAGUE_GOTHIC: Asset = asset!("/assets/League-Gothic.woff2");

fn main() {
    let _ = dioxus::logger::init(Level::INFO);

    init_tracking();

    dioxus::launch(App);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AppState {
    Home,
    Deck,
    Selection,
    Stats,
    Settings,
}

#[component]
fn Bar() -> Element {
    let settings = use_context::<Signal<Storable<Settings>>>();
    let mut state = use_context::<Signal<AppState>>();
    rsx! {
        div {
            class: "bar",
            if state() != AppState::Home {
                button {
                    class: "back",
                    onclick: move |_| {
                        if let AppState::Settings = *state.read() {
                            settings.read().save();
                        }
                        state.set(AppState::Home)
                    },
                    Icon {
                        icon: FaAngleLeft
                    }
                }
            }
            span {
                "Flashbang"
            }
        }
    }
}

#[component]
fn Home() -> Element {
    let mut state = use_context::<Signal<AppState>>();

    rsx! {
        div {
            class: "home",
            button {
                class: "study",
                onclick: move |_| state.set(AppState::Selection),
                Icon {
                    icon: fa::FaGraduationCap
                }
                span {
                    "Start"
                }
            }
            button {
                class: "stats",
                onclick: move |_| state.set(AppState::Stats),
                Icon {
                    icon: bs::BsGraphUp
                }
            }
            button {
                onclick: move |_| state.set(AppState::Settings),
                class: "settings",
                Icon {
                    icon: hi::HiCog
                }
            }
        }
    }
}

#[component]
fn Wrap(card_deck: Signal<Vec<CardHandle>>, width: ReadOnlySignal<u32>) -> Element {
    let settings: Signal<Storable<Settings>> = use_context();
    let repo = settings.read().repo.as_ref().cloned();
    let branch = settings.read().branch.clone();
    let token = settings.read().token.as_ref().cloned();
    // This can either be Ok(Vec<LoadError>), indicating that the loading succeeded with some recoverable errors
    // (we should display them, but keep the app running), or be Err(Box<dyn std::error::Error>), in which case
    // the loading failed.
    let loading_errors: MappedSignal<Result<Vec<LoadError>, Box<dyn std::error::Error>>> =
        use_resource(move || {
            let repo = repo.clone();
            let branch = branch.clone();
            let token = token.clone();
            async move {
                let Some(repo) = repo else {
                    init_store(CardStore::default());
                    return Ok(Vec::new());
                };
                let (store, errors) = CardStore::new_from_github(repo, branch, token).await?;
                init_store(store);
                Ok(errors)
            }
        })
        .suspend()?;

    // Throw if the loading failed
    loading_errors
        .read()
        .as_ref()
        .map_err(|e| CapturedError::from_display(e.to_string()))?;

    // Not memoed because this vec is empty almost all the time anyway
    let mut errors = use_signal(|| {
        loading_errors
            .read()
            .as_ref()
            .unwrap()
            .iter()
            .cloned()
            .map(CapturedError::from_display)
            .collect_vec()
    });

    let state: Signal<AppState> = use_context();
    rsx! {
        if !errors.is_empty() {
            ErrorDisplay {
                ui: Some(rsx! {
                    button {
                        class: "close",
                        onclick: move |_| errors.write().clear(),

                        "Recover"
                    }
                }),
                message: Some("Recoverable error encountered while loading cards".to_owned()),
                errors: errors
            }
        } else {
            if state() == AppState::Home {
                Home {}
            } else if state() == AppState::Deck {
                Deck {
                    cards: card_deck,
                    width: width()
                }
            } else if state() == AppState::Stats {
                Stats {}
            } else if state() == AppState::Selection {
                Selection {
                    deck: card_deck
                }
            } else if state() == AppState::Settings {
                SettingsComponent {}
            }
        }
    }
}

#[component]
fn ErrorDisplay(
    message: Option<String>,
    ui: Option<Element>,
    errors: ReadOnlySignal<Vec<CapturedError>>,
) -> Element {
    let message = message.unwrap_or_else(|| "An error was encountered.".to_owned());
    rsx! {
        div {
            class: "error",

            Icon {
                icon: MdError
            }
            span {
                class: "label",
                "{message}"
            }

            if let Some(ui) = ui {
                div {
                    class: "ui",
                    { ui }
                }
            }

            details {
                class: "debug",

                summary {
                    "More info"
                }

                div {
                    class: "more",

                    for err in errors.read().iter() {
                        span {
                            class: "item",
                            "{err}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn App() -> Element {
    let card_deck = use_signal(Vec::<CardHandle>::new);
    let mut width = use_signal(|| 0u32);

    let _settings = use_context_provider(|| {
        Signal::new(Storable::<Settings>::new("settings", Settings::default))
    });

    let mut state = use_context_provider(|| Signal::new(AppState::Home));

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Style {
            "
                @font-face {{
                    font-family: 'League Gothic';
                    font-style: normal;
                    font-weight: 400;
                    font-stretch: 100%;
                    font-display: swap;
                    src: url({FONT_LEAGUE_GOTHIC}) format('woff2');
                }}
                @font-face {{
                    font-family: 'Agbalumo';
                    font-style: normal;
                    font-weight: 400;
                    font-display: swap;
                    src: url({FONT_AGBALUMO}) format('woff2');
                }}
            "
        }
        Bar {}
        div {
            id: "app",
            tabindex: "0",
            onkeydown: move |event| {
                if event.key() == Key::GoBack && state() != AppState::Home {
                    event.prevent_default();
                    state.set(AppState::Home);
                }
            },
            onresize: move |event| width.set(event.get_content_box_size().map(|res| res.width as u32).unwrap_or(0)),
            ErrorBoundary {
                handle_error: move |error: ErrorContext| rsx! {
                    ErrorDisplay {
                        ui: error.show(),
                        errors: error.errors().to_vec(),
                    }
                },
                SuspenseBoundary {
                    fallback: |_| rsx! {
                        div {
                            class: "loader-wrap",
                            Icon {
                                class: "loader",
                                icon: LdLoaderCircle,
                            }

                            "Loading..."
                        }
                    },
                    Wrap {
                        card_deck,
                        width,
                    }
                }
            }
        }
        ToastDisplay {  }
    }
}
