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

pub const DEFAULT_TOAST_DURATION: u64 = 2;

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

pub struct Toaster {
    toasts: GlobalSignal<SlotMap<ToastHandle, Toast>>,
}

static TOASTER: Toaster = Toaster::new();

impl Toaster {
    const fn new() -> Self {
        Self {
            toasts: GlobalSignal::new(SlotMap::with_key),
        }
    }

    pub fn toast(message: String, duration: Duration) {
        let handle = TOASTER.toasts.write().insert_with_key(|handle| Toast {
            duration,
            handle,
            message,
            visible: true,
        });

        spawn_forever(async move {
            sleep(duration).await;
            if let Some(toast) = TOASTER.toasts.write().get_mut(handle) {
                toast.visible = false;
            }
            sleep(Duration::from_secs(1)).await;
            TOASTER.toasts.write().remove(handle);
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
    rsx! {
        div {
            class: "toast-wrap",

            for (handle, toast) in TOASTER.toasts.read().iter() {
                ShowToast {
                    key: "{handle:?}",
                    toast: toast.clone(),
                }
            }
        }
    }
}
