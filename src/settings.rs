use std::time::Duration;

use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::{
        fa_solid_icons::{FaFont, FaKey},
        go_icons::{GoGitBranch, GoNumber, GoRepo},
        md_hardware_icons::MdMemory,
    },
    Icon,
};

use crate::{
    algorithm,
    popup::{Toaster, DEFAULT_TOAST_DURATION},
    storage::{Serializable, Storable, TypeList},
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Settings {
    pub font_size: u32,
    pub retention: f32,
    pub repo: Option<String>,
    pub branch: String,
    pub token: Option<String>,
}

impl Serializable for Settings {
    type Fallback = TypeList<()>;
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_size: 10,
            retention: 0.8,
            branch: "main".to_owned(),
            repo: None,
            token: None,
        }
    }
}

#[component]
pub fn SettingsComponent() -> Element {
    let mut settings: Signal<Storable<Settings>> = use_context();

    rsx! {
        div {
            class: "settings",

            div {
                class: "item",
                Icon { icon: FaFont }
                span { class: "label", "Font size" }
                div {
                    class: "input",
                    input {
                        type: "range",
                        size: 1,
                        min: 10,
                        max: 25,
                        value: "{settings.read().font_size}",
                        oninput: move |e| {
                        settings.write().font_size = e.value().parse().unwrap_or_default();
                        }
                    }
                    div {
                        class: "extra",
                        "{settings.read().font_size}pt"
                    }
                }
            }

            div {
                class: "item",
                Icon { icon: GoRepo }
                span { class: "label", "Cards repo" }
                div {
                    class: "input",
                    input {
                        type: "text",
                        size: 1,
                        placeholder: "you/mycardsrepo",
                        value: r#"{settings.read().repo.as_ref().map(|s| s.as_str()).unwrap_or("")}"#,
                        onchange: move |e| {
                            let val = e.value();
                            if val.is_empty() {
                                settings.write().repo = None;
                            } else {
                                settings.write().repo = Some(val)
                            }
                            Toaster::toast("An app reload is necessary for this change to take effect.".to_owned(), Duration::from_secs(DEFAULT_TOAST_DURATION));
                        }
                    }
                }
            }

            div {
                class: "item",
                Icon { icon: FaKey }
                span { class: "label", "Github token" }
                div {
                    class: "input",
                    input {
                        type: "text",
                        size: 1,
                        placeholder: "your github token here",
                        value: r#"{settings.read().token.as_ref().map(|s| s.as_str()).unwrap_or("")}"#,
                        onchange: move |e| {
                            let val = e.value();
                            if val.is_empty() {
                                settings.write().token = None;
                            } else {
                                settings.write().token = Some(val)
                            }
                            Toaster::toast("An app reload is necessary for this change to take effect.".to_owned(), Duration::from_secs(DEFAULT_TOAST_DURATION));
                        }
                    }
                }
            }

            div {
                class: "item",
                Icon { icon: GoGitBranch }
                span { class: "label", "Repo branch" }
                div {
                    class: "input",
                    input {
                        class: "input",
                        size: 1,
                        type: "text",
                        value: "{settings.read().branch}",
                        onchange: move |e| {
                            settings.write().branch = e.value();
                            Toaster::toast("An app reload is necessary for this change to take effect.".to_owned(), Duration::from_secs(DEFAULT_TOAST_DURATION));
                        }
                    }
                }
            }

            div {
                class: "item",
                Icon { icon: MdMemory }
                span { class: "label", "FSRS retention" }
                div {
                    class: "input",
                    input {
                        class: "input",
                        type: "range",
                        min: 0,
                        max: 1,
                        step: 0.1,
                        size: 1,
                        value: "{settings.read().retention}",
                        oninput: move |e| {
                            settings.write().retention = e.value().parse().unwrap_or_default();
                        }
                    }
                    div {
                        class: "extra",
                        "{settings.read().retention:.1}"
                    }
                }
            }

            div {
                class: "item",
                Icon { icon: GoNumber }
                span { class: "label", "FSRS parameters"}
                div {
                    class: "input",
                    button {
                        class: "input",
                        onclick: move |_| {
                            if let Err(fsrs::FSRSError::NotEnoughData) = algorithm::update_params(None) {
                                Toaster::toast("Not enough card history to compute, come back when you've reviewed more".to_owned(), Duration::from_secs(DEFAULT_TOAST_DURATION));
                            }
                        },
                        "Compute"
                    }
                }
            }
        }
    }
}
