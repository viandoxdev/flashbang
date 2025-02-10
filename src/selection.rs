use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    iter::once,
    sync::Arc,
};

use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::fa_solid_icons::{FaChevronRight, FaFile, FaFolder},
    Icon,
};
use itertools::Itertools;
use rand::{seq::SliceRandom, SeedableRng};
use slotmap::{Key, SecondaryMap};
use smallvec::{smallvec, SmallVec};

use crate::{
    cards::{CardHandle, Tag},
    deck::STORE,
    popup::Popup,
    AppState,
};

struct Edges {
    children: Vec<Tag>,
    leaves: Vec<CardHandle>,
}

impl Edges {
    fn len(&self) -> usize {
        self.children.len() + self.leaves.len()
    }
}

struct TagTree {
    root: Tag,
    edges: HashMap<Tag, Edges>,
}

impl TagTree {
    /// Build tag tree from cards in store
    pub fn new() -> Self {
        let root = Tag::null();
        let mut edges = HashMap::new();

        edges.insert(
            root,
            Edges {
                children: vec![],
                leaves: vec![],
            },
        );

        let store = STORE.lock();

        for card in store.cards.values() {
            for path in &card.paths {
                // Add each tag to the tree
                let mut parent = root;

                for &tag in path.iter() {
                    let parent_edges = edges.get_mut(&parent).unwrap();
                    if !parent_edges.children.contains(&tag) {
                        parent_edges.children.push(tag);
                    }
                    if !edges.contains_key(&tag) {
                        edges.insert(
                            tag,
                            Edges {
                                leaves: vec![],
                                children: vec![],
                            },
                        );
                    }
                    parent = tag;
                }

                // Add the card
                edges
                    .get_mut(path.last().unwrap())
                    .unwrap()
                    .leaves
                    .push(card.handle);
            }
        }

        Self { root, edges }
    }

    fn iter_from(&self, from: Tag) -> ChildrenIterator<'_> {
        ChildrenIterator::new(self, from)
    }
}

struct ChildrenIterator<'a> {
    tree: &'a TagTree,
    stack: SmallVec<[(Tag, usize); 16]>,
}

impl<'a> ChildrenIterator<'a> {
    fn new(tree: &'a TagTree, node: Tag) -> Self {
        Self {
            tree,
            stack: smallvec![(node, 0)],
        }
    }
}

impl<'a> Iterator for ChildrenIterator<'a> {
    type Item = CardHandle;

    fn next(&mut self) -> Option<CardHandle> {
        let Some(&(mut top, mut index)) = self.stack.last() else {
            return None;
        };

        let mut edge = &self.tree.edges[&top];

        loop {
            let children = edge.children.len();
            if index < children {
                top = edge.children[index];
                index = 0;
                edge = &self.tree.edges[&top];
            } else if index - children < edge.leaves.len() {
                if let Some((_, index)) = self.stack.last_mut() {
                    *index += 1;
                }

                return Some(self.tree.edges[&top].leaves[index]);
            } else {
                self.stack.pop();
                if let Some((new_top, new_index)) = self.stack.last_mut() {
                    *new_index += 1;
                    top = *new_top;
                    index = *new_index;
                    edge = &self.tree.edges[&top];
                } else {
                    return None;
                }
            }
        }
    }
}

#[component]
pub fn ItemTag(tag: Tag, enabled: ReadOnlySignal<bool>, onactivate: EventHandler<bool>) -> Element {
    let mut pred = use_context::<Signal<Vec<Tag>>>();
    let mut parent = use_context::<Signal<Tag>>();

    let store = STORE.lock();
    let name = store.tags[tag].clone();

    rsx! {
        div {
            class: "item tag",
            div {
                class: "clickable",
                onclick: move |_| {
                    pred.write().push(parent());
                    parent.set(tag);
                },
                Icon {
                    icon: FaFolder
                }

                span {
                    class: "name",
                    "{name}"
                }
            }

            input {
                type: "checkbox",
                class: "check",
                checked: "{enabled}",
                onchange: move |event: Event<FormData>| {
                    onactivate(event.value() == "true")
                }
            }
        }
    }
}

#[component]
fn ItemCard(card: CardHandle, enabled: Signal<bool>) -> Element {
    let store = STORE.lock();
    let name = store.cards[card].name.clone();

    rsx! {
        div {
            class: "item card",
            onclick: move |_| {
                enabled.set(!enabled());
            },

            Icon {
                icon: FaFile
            }

            span {
                class: "name",
                "{name}"
            }

            input {
                class: "check",
                type: "checkbox",
                checked: "{enabled}"
            }
        }
    }
}

#[component]
pub fn Selection(deck: Signal<Vec<CardHandle>>) -> Element {
    let tree = Arc::new(TagTree::new());
    let root = tree.root;

    let mut pred = use_context_provider(|| Signal::new(Vec::<Tag>::new()));
    let mut parent = use_context_provider(|| Signal::new(tree.root));
    let mut empty_deck_popup = use_signal(|| false);

    let cards_enabled = Arc::new({
        let mut map = SecondaryMap::new();
        let store = STORE.lock();

        for card in store.cards.keys() {
            map.insert(card, use_signal(|| false));
        }

        map
    });

    let tags_enabled = Arc::new({
        let store = STORE.lock();
        store
            .tags
            .keys()
            .map(|tag| {
                let cards_enabled = cards_enabled.clone();
                let tree = tree.clone();
                (
                    tag,
                    use_memo(move || {
                        // Code is ass

                        // This is written as a for loop on purpose, we do NOT want the shortcircuitting
                        // nature of .any(), because this needs to be reactive
                        let mut on = false;
                        for card in tree.iter_from(tag) {
                            on = on || *(cards_enabled[card].read());
                        }
                        on
                    }),
                )
            })
            .collect::<SecondaryMap<_, _>>()
    });

    let pwd = use_memo(move || {
        let store = STORE.lock();
        let path_elements = pred
            .iter()
            .map(|t| *t)
            .chain(once(parent()))
            .enumerate()
            .skip(1)
            .map(|(i, t)| (i, t, store.tags[t].to_owned()));
        once((0, root, "Root".to_owned()))
            .chain(path_elements)
            .collect_vec()
    });

    // Some amount of randomness because we don't want the same decks ordered the same every time.
    let seed = use_resource(|| async move {
        let mut eval = document::eval(
            r#"
           dioxus.send(Math.random());
           "#,
        );
        eval.recv::<f64>().await.unwrap_or_default()
    });

    let enabled_count = {
        let cards_enabled = cards_enabled.clone();
        use_memo(move || {
            let mut count = 0;
            for s in cards_enabled.values() {
                if s() {
                    count += 1;
                }
            }
            count
        })
    };

    let start = {
        let cards_enabled = cards_enabled.clone();
        move |_| {
            if !cards_enabled.iter().any(|(_, s)| s()) {
                return;
            }

            // TODO: Find a cleaner way to seed this

            // We hash a bunch of random looking data together with a single u64 worth of
            // "entropy" if js eval returned before this is ran
            let mut hasher = DefaultHasher::new();
            let js_seed = seed().unwrap_or(0.0).to_bits();
            let mut cards = cards_enabled
                .iter()
                .filter_map(|(k, s)| s().then_some(k))
                .collect::<Vec<CardHandle>>();
            cards.iter().for_each(|t| t.hash(&mut hasher));
            js_seed.hash(&mut hasher);
            let seed = hasher.finish();
            let mut rng = rand::rngs::SmallRng::seed_from_u64(seed);
            cards.shuffle(&mut rng);

            deck.set(cards);

            let mut state: Signal<AppState> = use_context();
            state.set(AppState::Deck);
        }
    };

    rsx! {
        div {
            class: "selection",
            div {
                class: "header",
                for (index, tag, name) in pwd() {
                    Icon {
                        icon: FaChevronRight
                    }
                    button {
                        class: "tag",
                        onclick: move |_| {
                            parent.set(tag);
                            pred.write().resize(index, Tag::null());
                        },
                        "{name}"
                    }
                }

                div {
                    class: "spacer"
                }

                div {
                    class: "count",
                    "{enabled_count} selected"
                }
            }
            div {
                class: "items",

                for &tag in &tree.edges[&parent()].children {
                    ItemTag {
                        tag,
                        enabled: tags_enabled[tag],
                        onactivate: {
                            let cards_enabled = cards_enabled.clone();
                            let tree = tree.clone();
                            move |on| {
                                tree.iter_from(tag)
                                    .for_each(|c| {
                                        let mut sig = cards_enabled[c];
                                        sig.set(on)
                                    });
                            }
                        }
                    }
                }

                for &card in &tree.edges[&parent()].leaves {
                    ItemCard {
                        card,
                        enabled: cards_enabled[card]
                    }
                }
            }
            div {
                class: "footer",
                button {
                    class: if enabled_count() > 0 { "go" } else { "go locked" },
                    onclick: start,
                    "Start"
                }
            }

            if empty_deck_popup() {
                Popup {
                    span {
                        "You need to select at least one card to study !"
                    }
                    button {
                        class: "wide",
                        onclick: move |_| empty_deck_popup.set(false),
                        "Back"
                    }
                }
            }
        }
    }
}
