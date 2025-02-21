use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    iter::once,
    rc::Rc,
    time::Duration,
};

use dioxus::prelude::*;
use dioxus_free_icons::{
    icons::{
        fa_solid_icons::{FaChevronRight, FaFile, FaFolder},
        go_icons::GoCalendar,
        md_action_icons::MdFlipToBack,
        md_content_icons::MdSelectAll,
    },
    Icon,
};
use itertools::Itertools;
use rand::{seq::SliceRandom, SeedableRng};
use slotmap::{Key, SecondaryMap};
use smallvec::{smallvec, SmallVec};
use time::OffsetDateTime;

use crate::{
    cards::{CardHandle, Tag},
    deck::store,
    popup::{Toaster, DEFAULT_TOAST_DURATION},
    settings::Settings,
    storage::Storable,
    tracking::tracking,
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

        let store = store().lock();

        for card in store.cards.values() {
            for path in &card.paths {
                // Add each tag to the tree
                let mut parent = root;

                for &tag in path.iter() {
                    let parent_edges = edges.get_mut(&parent).unwrap();
                    if !parent_edges.children.contains(&tag) {
                        parent_edges.children.push(tag);
                    }
                    edges.entry(tag).or_insert_with(|| Edges {
                        leaves: Vec::new(),
                        children: Vec::new(),
                    });
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

impl Iterator for ChildrenIterator<'_> {
    type Item = CardHandle;

    fn next(&mut self) -> Option<CardHandle> {
        let &(mut top, mut index) = self.stack.last()?;

        let mut edge = &self.tree.edges[&top];

        loop {
            let children = edge.children.len();
            if index < children {
                top = edge.children[index];
                index = 0;
                edge = &self.tree.edges[&top];
                self.stack.push((top, index));
            } else if index - children < edge.leaves.len() {
                if let Some((_, index)) = self.stack.last_mut() {
                    *index += 1;
                }

                return Some(self.tree.edges[&top].leaves[index]);
            } else {
                self.stack.pop();
                let (new_top, new_index) = self.stack.last_mut()?;
                *new_index += 1;
                top = *new_top;
                index = *new_index;
                edge = &self.tree.edges[&top];
            }
        }
    }
}

#[component]
pub fn ItemTag(tag: Tag, enabled: ReadOnlySignal<bool>, onactivate: EventHandler<bool>) -> Element {
    let mut pred = use_context::<Signal<Vec<Tag>>>();
    let mut parent = use_context::<Signal<Tag>>();

    let store = store().lock();
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
    let store = store().lock();
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
    let settings: Signal<Storable<Settings>> = use_context();
    let tree = use_hook(|| Rc::new(TagTree::new()));
    let root = tree.root;

    let mut pred = use_context_provider(|| Signal::new(Vec::<Tag>::new()));
    let mut parent = use_context_provider(|| Signal::new(tree.root));

    let mut state: Signal<AppState> = use_context();

    let cards_enabled = use_hook(|| {
        Rc::new({
            let mut map = SecondaryMap::new();
            let store = store().lock();

            for card in store.cards.keys() {
                map.insert(card, Signal::new(false));
            }

            map
        })
    });

    let tags_enabled = use_hook(|| {
        Rc::new({
            let store = store().lock();
            store
                .tags
                .keys()
                .map(|tag| {
                    let cards_enabled = cards_enabled.clone();
                    let tree = tree.clone();
                    (
                        tag,
                        Memo::new(move || {
                            // Code is ass

                            // This is written as a for loop on purpose, we do NOT want the short circuiting
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
        })
    });

    let pwd = use_memo(move || {
        let store = store().lock();
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

            state.set(AppState::Deck);
        }
    };

    let select_all = {
        let cards_enabled = cards_enabled.clone();
        move |_| {
            // If any card is not selected, select all cards
            // Otherwise, deselect all cards
            let should_select = cards_enabled.iter().any(|(_, s)| !s());

            for &signal in cards_enabled.values() {
                // Goofy but needs to be mut
                let mut signal = signal;
                signal.set(should_select);
            }
        }
    };

    let flip_all = {
        let cards_enabled = cards_enabled.clone();
        move |_| {
            // Invert the selection state of each card
            for &signal in cards_enabled.values() {
                let mut signal = signal;
                signal.set(!signal());
            }
        }
    };

    let select_due = {
        let cards_enabled = cards_enabled.clone();
        move |_| {
            let tracking = tracking().lock();
            let store = store().lock();
            let now = OffsetDateTime::now_utc().date();
            let mut count = 0usize;

            for (key, card) in &store.cards {
                let due_date = tracking
                    .cards_info
                    .get(&card.id)
                    .and_then(|info| info.due)
                    .and_then(|due| OffsetDateTime::from_unix_timestamp(due).ok())
                    .map(OffsetDateTime::date);
                let is_due = due_date.map(|date| date == now).unwrap_or(false);
                let mut card = cards_enabled[key];

                card.set(is_due);
                if is_due {
                    count += 1;
                }
            }

            if count == 0 {
                Toaster::toast(
                    "No cards due today, you can choose some by hand instead.".to_owned(),
                    Duration::from_secs(DEFAULT_TOAST_DURATION),
                );
            }
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

                button {
                    class: "selection",
                    onclick: select_due,
                    Icon {
                        icon: GoCalendar,
                    }
                }

                button {
                    class: "selection",
                    onclick: select_all,
                    Icon {
                        icon: MdSelectAll
                    }
                }

                button {
                    class: "selection",
                    onclick: flip_all,
                    Icon {
                        icon: MdFlipToBack
                    }
                }

                div {
                    class: "count",
                    "{enabled_count} selected"
                }
            }
            if settings.read().repo.is_some() {
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
            } else {
                div {
                    class: "placeholder",

                    div {
                        "No cards are available because the card repository isn't set in settings."
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
        }
    }
}
