use deck::Deck;
use dioxus::{logger::tracing::Level, prelude::*};
use dioxus_free_icons::{
    icons::{
        bs_icons as bs,
        fa_solid_icons::{self as fa, FaAngleLeft},
        hi_solid_icons as hi,
    },
    Icon,
};
use dioxus_sdk::{
    set_dir,
    storage::{use_storage, LocalStorage, SessionStorage},
};
use selection::Selection;
use settings::{Settings, SettingsComponent};

mod cards;
mod deck;
mod selection;
mod settings;
mod typst_wrap;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const FONT_AGBALUMO: Asset = asset!("/assets/Agbalumo.woff2");
const FONT_LEAGUE_GOTHIC: Asset = asset!("/assets/League-Gothic.woff2");

#[cfg(not(any(target_os = "android", target_os = "linux")))]
type Storage = SessionStorage;
#[cfg(any(target_os = "android", target_os = "linux"))]
type Storage = LocalStorage;

fn main() {
    let _ = dioxus::logger::init(Level::INFO);

    // Ugly but no api in dioxus yet
    #[cfg(target_os = "android")]
    set_dir!("/data/user/0/dev.vndx.Flashbang/files");
    #[cfg(target_os = "linux")]
    set_dir!();

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
fn Stats() -> Element {
    rsx! {
        div {
            class: "stats",
            "WIP (STATS)"
        }
    }
}

#[component]
fn Bar() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    rsx! {
        div {
            class: "bar",
            if state() != AppState::Home {
                button {
                    class: "back",
                    onclick: move |_| state.set(AppState::Home),
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
    let store = deck::STORE.lock();

    let card_deck = use_signal(|| vec![]);
    let mut width = use_signal(|| 0u32);

    let mut settings = use_storage::<Storage, _>("settings".to_owned(), Settings::default);
    let mut settings = use_context_provider(|| settings);

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
