use std::time::Duration;

use dioxus::prelude::*;
use slotmap::{new_key_type, SlotMap};

use async_std::task::sleep;

// TODO: Remove this if unnecessary
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

new_key_type! {
    struct ToastHandle;
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct Toast {
    handle: ToastHandle,
    message: String,
    duration: Duration,
    visible: bool,
}

#[derive(Clone, Copy)]
pub struct Toaster {
    toasts: Signal<SlotMap<ToastHandle, Toast>>,
}

impl Toaster {
    /// Create a new toaster, this creates a signal so this should be called from a component
    pub fn new() -> Self {
        Self {
            toasts: Signal::new(SlotMap::with_key()),
        }
    }

    pub fn clear(&self) {
        let mut toasts = self.toasts;
        toasts.write().clear();
    }

    pub fn toast(&self, message: String, duration: Duration) {
        let mut toasts = self.toasts;
        let handle = toasts.write().insert_with_key(|handle| Toast {
            duration,
            handle,
            message,
            visible: true,
        });

        spawn_forever(async move {
            sleep(duration).await;
            if let Some(toast) = toasts.write().get_mut(handle) {
                toast.visible = false;
            }
            sleep(Duration::from_secs(1)).await;
            toasts.write().remove(handle);
        });
    }
}

#[component]
fn ShowToast(toast: Toast) -> Element {
    let opacity_val = if toast.visible { "1.0" } else { "0.0" };
    let height = if toast.visible { "4em" } else { "0em" };
    rsx! {
        div {
            class: "toast",
            opacity: "{opacity_val}",
            max_height: "{height}",
            "{toast.message}"
        }
    }
}

#[component]
pub fn ToastDisplay() -> Element {
    let toaster: Toaster = use_context();
    rsx! {
        div {
            class: "toast-wrap",

            for (_handle, toast) in toaster.toasts.read().iter() {
                ShowToast {
                    key: _handle,
                    toast: toast.clone(),
                }
            }
        }
    }
}
