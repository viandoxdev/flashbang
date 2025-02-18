#![allow(dead_code)]

use cards::CardStore;
use deck::{init_store, Deck};
use dioxus::{logger::tracing::Level, prelude::*};
use dioxus_free_icons::{
    icons::{
        bs_icons as bs,
        fa_solid_icons::{self as fa, FaAngleLeft},
        hi_solid_icons as hi,
        md_alert_icons::MdError,
    },
    Icon,
};

use popup::{ToastDisplay, Toaster};
use selection::Selection;
use settings::{Settings, SettingsComponent};
use stats::Stats;
use storage::Storable;
use tracking::init_tracking;

mod algorithm;
mod cards;
mod deck;
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

    init_store(|| {
        let mut store = CardStore::default();
        store
            .load(include_str!("../test.typ"))
            .expect("Failure on load");
        store
    });

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
fn App() -> Element {
    let card_deck = use_signal(Vec::new);
    let mut width = use_signal(|| 0u32);

    let _settings = use_context_provider(|| {
        Signal::new(Storable::<Settings>::new("settings", Settings::default))
    });

    let mut state = use_context_provider(|| Signal::new(AppState::Home));
    let toaster = use_root_context(Toaster::new);

    use_effect(use_reactive((&state,), move |_| toaster.clear()));

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
                handle_error: move |error: ErrorContext| {
                    rsx! {
                        div {
                            class: "error",

                            Icon {
                                icon: MdError
                            }
                            span {
                                class: "label",
                                "An error was encountered."
                            }

                            if let Some(ui) = error.show() {
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

                                    for err in error.errors() {
                                        span {
                                            class: "item",
                                            "{err}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
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
        ToastDisplay {  }
    }
}
