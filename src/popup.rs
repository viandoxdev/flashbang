use dioxus::prelude::*;

#[component]
pub fn Popup(children: Element) -> Element {
    rsx! {
        div {
            class: "popup-wrap",

            div {
                class: "popup",

                {children}
            }
        }
    }
}
