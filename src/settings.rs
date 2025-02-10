use dioxus::prelude::*;
use dioxus_free_icons::{icons::fa_solid_icons::FaFont, Icon};

use crate::storage::{Serializable, Storable, TypeList};

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct Settings {
    pub font_size: u32,
}

impl Serializable for Settings {
    type Fallback = TypeList<()>;
}

impl Default for Settings {
    fn default() -> Self {
        Self { font_size: 10 }
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
        }
    }
}
