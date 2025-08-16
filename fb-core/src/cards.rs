//! Contains logic related to cards:
//!  - parsing
//!  - splitting
//!  - building typst source files

use std::error::Error;
use std::fmt::Display;
use std::sync::Arc;
use std::usize;
use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use itertools::Itertools;
use nucleo::{
    Injector, Nucleo,
    pattern::{CaseMatching, Normalization},
};
use parking_lot::Mutex;

use crate::arc_struct;

arc_struct! {
    pub struct Tag {
        inner TagInner {
            name: String,
            full_path: String,
        }

        state TagState {
            children: HashSet<Tag>,
            cards: HashSet<CardWeak>,
            indirect_cards: HashSet<CardWeak>,
        }
    }

    struct Header {
        inner HeaderInner {
            content: String
        }
    }

    pub struct Card {
        weak CardWeak;

        inner CardInner {
            pub id: String,
        }

        state CardState {
            pub name: String,
            pub locations: Vec<Tag>,
            header: Option<Header>,
            /// Typst source for the question
            question: String,
            /// Typst source for the answer
            answer: String,
        }
    }
}

impl Tag {
    fn new(full_path: &str, store: &mut CardStore) -> Self {
        // Find the index of the last '.' character
        // The tag's name is str[index..], the rest is there to make sure we can
        // distinguish between to tags of the same name under different paths :
        //  Maths.Algebra.Theorems != Maths.Calculus.Theorems
        let index = full_path
            .char_indices()
            .rev()
            .find_map(|(i, c)| (c == '.').then_some(i + 1))
            .unwrap_or(0);
        let inner = TagInner {
            name: full_path[index..].to_owned(),
            full_path: full_path.to_owned(),
            state: Mutex::new(TagState {
                children: HashSet::new(),
                cards: HashSet::new(),
                indirect_cards: HashSet::new(),
            }),
        };

        let tag = Tag(Arc::new(inner));

        if let Some(parent) = tag.parent(store) {
            parent.add_child(tag.clone());
        }

        let root = tag.root(store);

        store.roots.insert(root);

        tag
    }

    fn ancestors(&self, store: &mut CardStore) -> impl Iterator<Item = Tag> {
        self.full_path
            .char_indices()
            .filter(|&(_, c)| c == '.')
            .map(|(i, _)| store.tag(&self.full_path[0..i]))
    }

    fn root(&self, store: &mut CardStore) -> Tag {
        self.ancestors(store).next().unwrap_or_else(|| self.clone())
    }

    fn parent(&self, store: &mut CardStore) -> Option<Tag> {
        self.ancestors(store).last()
    }

    fn add_child(&self, child: Tag) {
        self.state.lock().children.insert(child);
    }

    fn add_card(&self, card: &Card) {
        self.state.lock().cards.insert(card.downgrade());
        self.state.lock().indirect_cards.insert(card.downgrade());
    }

    fn add_card_indirect(&self, card: &Card) {
        self.state.lock().indirect_cards.insert(card.downgrade());
    }
}

impl Header {
    pub fn new(content: &str) -> Self {
        Header(Arc::new(HeaderInner {
            content: content.to_owned(),
        }))
    }
}

impl Card {
    fn new(
        id: String,
        name: String,
        header: Option<Header>,
        locations: Vec<Tag>,
        question: String,
        answer: String,
    ) -> Self {
        Card(Arc::new(CardInner {
            id,
            state: Mutex::new(CardState {
                header,
                name: name.to_owned(),
                locations,
                question: question.to_owned(),
                answer: answer.to_owned(),
            }),
        }))
    }
}

/// Rating of how well a card was answered
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, uniffi::Enum,
)]
pub enum Rating {
    Again,
    Hard,
    Good,
    Easy,
}

impl Rating {
    pub fn str(&self) -> &'static str {
        match self {
            Rating::Again => "Again",
            Rating::Hard => "Hard",
            Rating::Good => "Good",
            Rating::Easy => "Easy",
        }
    }
}

impl Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str())
    }
}

/// Global store for cards and tags
#[derive(Clone, uniffi::Object)]
pub struct CardStore {
    tags_map: HashMap<String, Tag>,
    /// Fuzzy finding utilities
    nucleo: Arc<Mutex<Nucleo<Card>>>,
    injector: Arc<Injector<Card>>,
    /// Garbage collection for cards (when reloading)
    garbage: HashSet<String>,
    /// Set of root tags
    roots: HashSet<Tag>,
    /// Sha of the commit this store was based off
    pub sha: String,
    /// Cards
    pub cards: HashMap<String, Card>,
}

impl Default for CardStore {
    fn default() -> Self {
        let nucleo = Nucleo::<Card>::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 1);
        let injector = nucleo.injector();

        Self {
            nucleo: Arc::new(Mutex::new(nucleo)),
            injector: Arc::new(injector),
            sha: "".to_owned(),
            tags_map: HashMap::new(),
            cards: HashMap::new(),
            garbage: HashSet::new(),
            roots: HashSet::new(),
        }
    }
}

/// Config for things that the source needs to compile
#[derive(Clone, Copy, uniffi::Record)]
pub struct SourceConfig {
    /// Page width in pt
    pub page_width: u32,
    // Text size in pt
    pub text_size: u32,
}

#[derive(Debug, Clone, Copy, uniffi::Enum)]
pub enum FuzzyStatus {
    Stale,
    Updated,
    Finished,
}

impl From<nucleo::Status> for FuzzyStatus {
    fn from(value: nucleo::Status) -> Self {
        if !value.running {
            Self::Finished
        } else if value.changed {
            Self::Updated
        } else {
            Self::Stale
        }
    }
}

impl CardStore {
    /// Get (potentially store) a tag from its string path
    /// str must be the full path of the tag:
    ///     "Maths.Algebra.Theorems" for the "Theorems" tag
    fn tag(&mut self, str: &str) -> Tag {
        if let Some(tag) = self.tags_map.get(str) {
            tag.clone()
        } else {
            let tag = Tag::new(str, self);

            self.tags_map.insert(str.to_owned(), tag.clone());
            tag
        }
    }

    /// Build the source for a set of cards and a config
    pub fn build_source(
        &self,
        cards: impl IntoIterator<Item = Card>,
        config: SourceConfig,
    ) -> Result<String, std::io::Error> {
        use std::io::Write;

        let mut w = Vec::new();
        let mut last_header = None;

        writeln!(&mut w, "#import \"cards_internal.typ\": *")?;
        writeln!(&mut w, "#show: setup")?;
        writeln!(&mut w, "#set page(width: {}pt)", config.page_width)?;
        writeln!(&mut w, "#set text(size: {}pt)", config.text_size)?;
        writeln!(&mut w, "#[")?;

        for card in cards {
            let state = card.state.lock();

            if last_header != state.header {
                writeln!(&mut w, "]")?;
                writeln!(&mut w, "#[")?;

                if let Some(header) = (&state.header).as_ref() {
                    writeln!(&mut w, "{}", header.content)?;
                }

                last_header = state.header.clone();
            }

            write!(&mut w, "#card(\"{}\", \"{}\", (", card.id, state.name)?;
            for tag in &state.locations {
                write!(&mut w, "\"{}\",", tag.full_path)?;
            }
            writeln!(&mut w, "))")?;

            write!(&mut w, "{}", state.question)?;
            writeln!(&mut w, "#answer")?;
            write!(&mut w, "{}", state.answer)?;
        }

        String::from_utf8(w).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "Couldn't parse bytes to string")
        })
    }

    /// Mark as cards as unused, for the next collect_garbage call. This is reset for cards that
    /// are loaded (see load)
    pub fn mark_cards_for_garbage_collection(&mut self) {
        self.garbage.extend(self.cards.keys().cloned());
    }

    /// Load a typst card source file into the store
    ///
    /// For reloads, use with garbage collection to avoid keeping removed cards in memmory:
    /// ```rust
    /// store.mark_cards_for_garbage_collection();
    /// for content in files {
    ///     store.load(content);
    /// }
    /// // This removes the cards that weren't reloaded from the store
    /// store.collect_garbage()
    /// ```
    pub fn load<'a>(&mut self, content: &'a str) -> Result<(), Box<dyn Error + 'a>> {
        if content.starts_with("//![FLASHBANG IGNORE]")
            || content.starts_with("//![FLASHBANG INCLUDE]")
        {
            return Ok(());
        }

        use nom::{
            IResult, Parser, bytes::*, character::*, combinator::*, error::*, multi::*, sequence::*,
        };

        // Parsing stuff

        pub fn ws<'a, O, E: ParseError<&'a str>, F>(
            inner: F,
        ) -> impl Parser<&'a str, Output = O, Error = E>
        where
            F: Parser<&'a str, Output = O, Error = E>,
        {
            delimited(multispace0(), inner, multispace0())
        }

        fn string(input: &str) -> IResult<&str, &str> {
            let (input, (_, content, _)) = (tag("\""), take_until("\""), tag("\"")).parse(input)?;
            Ok((input, content))
        }

        fn card_header(input: &str) -> IResult<&str, (&str, &str, Vec<&str>)> {
            let (input, _) = (tag("#"), ws(tag("card"))).parse(input)?;
            let (input, (_, id, _)) = (tag("("), ws(string), tag(",")).parse(input)?;
            let (input, (name, _)) = (ws(string), tag(",")).parse(input)?;
            let (input, locations) = delimited(
                ws(tag("(")),
                terminated(separated_list0(ws(tag(",")), string), opt(ws(tag(",")))),
                ws(tag(")")),
            )
            .parse(input)?;
            let (input, _) = ws(tag(")")).parse(input)?;
            Ok((input, (id, name, locations)))
        }

        fn card(input: &str) -> IResult<&str, (&str, &str, Vec<&str>, &str, &str)> {
            let (input, (id, name, locations)) = card_header(input)?;
            let (input, (question, _)) = (take_until("#answer"), tag("#answer")).parse(input)?;
            let (input, answer) = take_until::<&str, &str, Error<&str>>("#card")
                .parse(input)
                .unwrap_or(("", input));
            Ok((input, (id, name, locations, question, answer)))
        }

        // Skip before header (if any)
        let (content, has_header) =
            match take_until::<&str, &str, Error<&str>>("#card").parse(content) {
                // Found header section
                Ok((content, _)) => (content, true),
                // No header section, continue as usual
                _ => (content, false),
            };

        // Parse header
        let Ok((content, header)) = take_until::<&str, &str, Error<&str>>("#card").parse(content)
        else {
            // No card in file
            return Ok(());
        };

        // Save header
        let header = if !header.is_empty() && has_header {
            Some(Header::new(header))
        } else {
            None
        };

        // Parse cards
        let (_, (cards, _)) = many_till(card, eof).parse(content)?;

        // Resolve tags and names
        for (id, name, locations, question, answer) in cards {
            let locations = locations.into_iter().map(|t| self.tag(t)).collect_vec();
            let id = id.to_owned();

            self.garbage.remove(&id);

            let card = Card::new(
                id.clone(),
                name.to_owned(),
                header.clone(),
                locations.clone(),
                question.to_owned(),
                answer.to_owned(),
            );

            // Register card in tags
            for tag in &locations {
                tag.add_card(&card);

                for ancestor in tag.ancestors(self) {
                    ancestor.add_card_indirect(&card);
                }
            }

            self.cards.insert(id, card.clone());
            self.injector.push(card, |_, row| row[0] = name.into());
        }

        Ok(())
    }

    pub fn cards(&self) -> Vec<Card> {
        self.cards.values().cloned().collect_vec()
    }

    pub fn roots(&self) -> Vec<Tag> {
        self.roots.iter().cloned().collect_vec()
    }

    /// Remove all cards that were marked for garbage collection.
    pub fn collect_garbage(&mut self) {
        for id in self.garbage.drain() {
            self.cards.remove(&id);
        }
    }

    pub fn fuzzy_init(&self, pattern: &str) {
        self.nucleo.lock().pattern.reparse(
            0,
            pattern,
            CaseMatching::Ignore,
            Normalization::Smart,
            false,
        );
    }

    pub fn fuzzy_tick(&self) -> FuzzyStatus {
        self.nucleo.lock().tick(500).into()
    }

    pub fn fuzzy_results(&self) -> Vec<Card> {
        self.nucleo
            .lock()
            .snapshot()
            .matched_items(..)
            .map(|item| item.data.clone())
            .collect_vec()
    }
}

#[uniffi::export]
impl TagInner {
    #[uniffi::method(name = "name")]
    pub fn _name(&self) -> String {
        self.name.clone()
    }
    #[uniffi::method(name = "fullPath")]
    pub fn _full_path(&self) -> String {
        self.full_path.clone()
    }
    #[uniffi::method(name = "cards")]
    pub fn _cards(&self) -> Vec<Card> {
        self.state
            .lock()
            .cards
            .iter()
            .filter_map(|w| w.upgrade())
            .collect_vec()
    }
    #[uniffi::method(name = "indirectCards")]
    pub fn _indirect_cards(&self) -> Vec<Card> {
        self.state
            .lock()
            .indirect_cards
            .iter()
            .filter_map(|w| w.upgrade())
            .collect_vec()
    }
    #[uniffi::method(name = "children")]
    pub fn _children(&self) -> Vec<Tag> {
        self.state.lock().children.iter().cloned().collect_vec()
    }
}

#[uniffi::export]
impl CardInner {
    #[uniffi::method(name = "name")]
    pub fn _name(&self) -> String {
        self.state.lock().name.clone()
    }
    #[uniffi::method(name = "id")]
    pub fn _id(&self) -> String {
        self.id.clone()
    }
    #[uniffi::method(name = "paths")]
    pub fn _locations(&self) -> Vec<Tag> {
        self.state.lock().locations.clone()
    }
}
