use std::time::Duration;

use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::{fa_solid_icons::FaFont, go_icons::GoNumber},
    Icon,
};

use crate::{
    algorithm,
    popup::Toaster,
    storage::{Serializable, Storable, TypeList},
};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Settings {
    pub font_size: u32,
    pub retention: f32,
}

impl Serializable for Settings {
    type Fallback = TypeList<()>;
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            font_size: 10,
            retention: 0.8,
        }
    }
}

#[component]
pub fn SettingsComponent() -> Element {
    let mut settings: Signal<Storable<Settings>> = use_context();
    let toaster: Toaster = use_context();

    rsx! {
        div {
            class: "settings",

            div {
                class: "item",

                Icon { icon: FaFont }
                span { class: "label", "Font size" }
                input {
                    type: "range",
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

            div {
                class: "item",

                Icon { icon: GoNumber }
                span { class: "label", "FSRS parameters"}

                button {
                    onclick: move |_| {
                        if let Err(fsrs::FSRSError::NotEnoughData) = algorithm::update_params(None) {
                            toaster.toast("Not enough card history to compute, come back when you've reviewed more".to_owned(), Duration::from_secs(3));
                        }
                    },

                    "Compute"
                }
            }
        }
    }
}
