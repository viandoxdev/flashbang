//! Contains logic related to cards:
//!  - parsing
//!  - splitting
//!  - building typst source files

use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::iter::once;
use std::ops::Deref;
use std::sync::Arc;
use std::usize;

use itertools::Itertools;
use nucleo::{
    Injector, Nucleo,
    pattern::{CaseMatching, Normalization},
};
use parking_lot::Mutex;

use crate::new_type_index;

new_type_index!(Tag);
new_type_index!(CardHandle);
new_type_index!(HeaderHandle);

uniffi::custom_newtype!(CardPath, Vec<Tag>);

/// Cards have a set of paths: different locations they may be found at.
/// These are sequences of tags (i.e Math.Algebra.Field)
#[derive(Clone, Debug)]
pub struct CardPath(Vec<Tag>);

impl Deref for CardPath {
    type Target = Vec<Tag>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl CardPath {
    fn from_str(s: &str, store: &mut CardStore) -> Self {
        Self(
            // Add a . at the end of the iterator
            (s.char_indices().chain(once((s.len(), '.'))))
                .filter_map(|(i, c)| (c == '.').then_some(i)) // keep the positions of '.'s
                .map(|i| store.tag(&s[..i])) // Grab views of the string: 'Maths', 'Maths.Algebra', ... and get the tag
                .collect(),
        )
    }
}

/// Represents a card's content
#[derive(Clone, Debug, uniffi::Object)]
pub struct Card {
    pub name: String,
    pub id: String,
    pub paths: Vec<CardPath>,
    header: Option<HeaderHandle>,
    /// Typst source for the question
    question: String,
    /// Typst source for the answer
    answer: String,
    /// Its own handle in the CardStore
    pub handle: CardHandle,
}

#[uniffi::export]
impl Card {
    #[uniffi::method(name = "name")]
    pub fn _name(&self) -> String {
        self.name.clone()
    }
    #[uniffi::method(name = "id")]
    pub fn _id(&self) -> String {
        self.id.clone()
    }
    #[uniffi::method(name = "paths")]
    pub fn _paths(&self) -> Vec<CardPath> {
        self.paths.clone()
    }
    #[uniffi::method(name = "handle")]
    pub fn _handle(&self) -> CardHandle {
        self.handle
    }
}

/// Rating of how well a card was answered
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
    nucleo: Arc<Mutex<Nucleo<CardHandle>>>,
    injector: Arc<Injector<CardHandle>>,
    /// Sha of the commit this store was based off
    pub sha: String,
    /// Name of tags
    pub tags: Vec<String>,
    /// Cards
    pub cards: Vec<Card>,
    pub headers: Vec<String>,
}

impl Default for CardStore {
    fn default() -> Self {
        let nucleo = Nucleo::<CardHandle>::new(nucleo::Config::DEFAULT, Arc::new(|| {}), None, 1);
        let injector = nucleo.injector();

        Self {
            nucleo: Arc::new(Mutex::new(nucleo)),
            injector: Arc::new(injector),
            sha: "".to_owned(),
            tags_map: HashMap::new(),
            tags: Vec::new(),
            cards: Vec::new(),
            headers: Vec::new(),
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

#[uniffi::export]
impl CardStore {
    #[uniffi::method(name = "cards")]
    pub fn _cards(&self) -> Vec<Arc<Card>> {
        self.cards.iter().map(|c| Arc::new(c.clone())).collect_vec()
    }

    #[uniffi::method(name = "tags")]
    pub fn _tags(&self) -> Vec<String> {
        self.tags.clone()
    }
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
    pub fn clear(&mut self) {
        self.nucleo.lock().restart(true);
        self.injector = Arc::new(self.nucleo.lock().injector());
        self.sha = "".to_string();
        self.tags.clear();
        self.cards.clear();
        self.headers.clear();
        self.tags_map.clear();
    }

    /// Get (potentially store) a tag from its string path
    /// str must be the full path of the tag:
    ///     "Maths.Algebra.Theorems" for the "Theorems" tag
    fn tag(&mut self, str: &str) -> Tag {
        if let Some(&tag) = self.tags_map.get(str) {
            tag
        } else {
            // Find the index of the last '.' character
            // The tag's name is str[index..], the rest is there to make sure we can
            // distinguish between to tags of the same name under different paths :
            //  Maths.Algebra.Theorems != Maths.Calculus.Theorems
            let index = str
                .char_indices()
                .rev()
                .filter_map(|(i, c)| (c == '.').then_some(i + 1))
                .next()
                .unwrap_or(0);

            let tag = Tag(self.tags.len() as u64);
            self.tags.push(str[index..].to_owned());
            self.tags_map.insert(str.to_owned(), tag);
            tag
        }
    }

    /// Build the source for a set of cards and a config
    pub fn build_source(
        &self,
        cards: impl IntoIterator<Item = CardHandle>,
        config: SourceConfig,
    ) -> Result<String, std::io::Error> {
        use std::io::Write;

        const NO_HEADER: Option<HeaderHandle> = Some(HeaderHandle(usize::MAX as u64));

        let mut w = Vec::new();
        let mut last_header = NO_HEADER;
        writeln!(&mut w, "#import \"cards_internal.typ\": *")?;
        writeln!(&mut w, "#show: setup")?;
        writeln!(&mut w, "#set page(width: {}pt)", config.page_width)?;
        writeln!(&mut w, "#set text(size: {}pt)", config.text_size)?;
        for card in cards {
            let card = &self.cards[card.index()];

            if last_header != card.header {
                if last_header != NO_HEADER {
                    writeln!(&mut w, "]")?;
                }
                writeln!(&mut w, "#[")?;
                if let Some(handle) = card.header {
                    writeln!(&mut w, "{}", self.headers[handle.index()])?;
                }

                last_header = card.header;
            }

            write!(&mut w, "#card(\"{}\", \"{}\", (", card.id, card.name)?;
            for path in &card.paths {
                write!(&mut w, "\"")?;
                for chunk in
                    Itertools::intersperse(path.iter().map(|&t| self.tags[t.index()].as_str()), ".")
                {
                    write!(&mut w, "{}", chunk)?;
                }
                write!(&mut w, "\",")?;
            }
            writeln!(&mut w, "))")?;

            write!(&mut w, "{}", card.question)?;
            writeln!(&mut w, "#answer")?;
            write!(&mut w, "{}", card.answer)?;
        }

        String::from_utf8(w).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "Couldn't parse bytes to string")
        })
    }

    /// Load a typst card source file into the store
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
            let (input, paths) = delimited(
                ws(tag("(")),
                terminated(separated_list0(ws(tag(",")), string), opt(ws(tag(",")))),
                ws(tag(")")),
            )
            .parse(input)?;
            let (input, _) = ws(tag(")")).parse(input)?;
            Ok((input, (id, name, paths)))
        }

        fn card(input: &str) -> IResult<&str, (&str, &str, Vec<&str>, &str, &str)> {
            let (input, (id, name, paths)) = card_header(input)?;
            let (input, (question, _)) = (take_until("#answer"), tag("#answer")).parse(input)?;
            let (input, answer) = take_until::<&str, &str, Error<&str>>("#card")
                .parse(input)
                .unwrap_or(("", input));
            Ok((input, (id, name, paths, question, answer)))
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
            let index = self.headers.len();
            self.headers.push(header.to_string());
            Some(index.into())
        } else {
            None
        };

        // Parse cards
        let (_, (cards, _)) = many_till(card, eof).parse(content)?;

        // Resolve tags, names, handles
        for (id, name, paths, question, answer) in cards {
            let paths = paths
                .into_iter()
                .map(|s| CardPath::from_str(s, self))
                .collect();

            let handle: CardHandle = self.cards.len().into();

            self.cards.push(Card {
                header,
                id: id.to_owned(),
                name: name.to_owned(),
                paths,
                question: question.to_owned(),
                answer: answer.to_owned(),
                handle,
            });

            self.injector.push(handle, |_, row| row[0] = name.into());
        }

        Ok(())
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

    pub fn fuzzy_results(&self) -> Vec<CardHandle> {
        self.nucleo.lock().snapshot().matched_items(..).map(|item| *item.data).collect_vec()
    }
}
