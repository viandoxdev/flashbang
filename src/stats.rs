use dioxus::prelude::*;
use itertools::Itertools;
use time::{macros::format_description, OffsetDateTime};

use crate::tracking::tracking;

fn timestamp_to_string(timestamp: i64) -> String {
    const FORMAT: &[time::format_description::BorrowedFormatItem<'_>] =
        format_description!("[weekday repr:short], [day] [month repr:short] [year]");
    OffsetDateTime::from_unix_timestamp(timestamp)
        .ok()
        .and_then(|d| d.format(FORMAT).ok())
        .unwrap_or_else(|| "???".to_owned())
}

#[component]
pub fn Stats() -> Element {
    let (sessions, cards) = {
        let track = tracking().lock();
        let sessions = track
            .sessions
            .iter()
            .map(|s| (timestamp_to_string(s.timestamp), s.score, s.size))
            .collect_vec();
        let cards = track
            .cards_info
            .iter()
            .map(|(k, c)| {
                (
                    k.to_owned(),
                    match c.memory_state {
                        Some(state) => {
                            format!("{:.02}s {:.02}d", state.stability, state.difficulty)
                        }
                        None => "No reviews".to_owned(),
                    },
                    c.due
                        .map(timestamp_to_string)
                        .unwrap_or_else(|| "Not due".to_owned()),
                    c.reviews
                        .iter()
                        .map(|v| (timestamp_to_string(v.timestamp), v.rating))
                        .collect_vec(),
                )
            })
            .collect_vec();

        (sessions, cards)
    };
    rsx! {
        div {
            class: "stats",
            div {
                class: "sessions",

                for (timestamp, _, size) in &sessions {
                    div {
                        class: "session",

                        span { "on " }
                        span { class: "timestamp", "{timestamp}" }
                        span { ", you studied " }
                        span { class: "size", "{size}" }
                        span { " cards." }
                    }
                }
            }

            div {
                class: "cards",

                for (id, state, due, views) in &cards {
                    div {
                        class: "card",

                        div { class: "id", "{id}" }
                        div { class: "score", "{state}"}
                        div { class: "label", "Due" }
                        div { class: "due", "{due}" }

                        div {
                            class: "views",
                            for (timestamp, rating) in &views {
                                div {
                                    class: "view",

                                    div { class: "timestamp", "{timestamp}"}
                                    div { class: "rating", "{rating}"}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
