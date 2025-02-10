//! Contains logic related to cards:
//!  - parsing
//!  - splitting
//!  - building typst source files

use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use std::iter::once;
use std::ops::Deref;

use itertools::Itertools;
use slotmap::SlotMap;
use smallvec::SmallVec;

slotmap::new_key_type! {
    /// Handle to a tag, akin to a directory, has a name held in CardStore
    pub struct Tag;
    /// Handle to a card (held in CardStore)
    pub struct CardHandle;
}

/// Cards have a set of paths: different locations they may be found at.
/// These are sequences of tags (i.e Math.Algebra.Field)
#[derive(Debug, Clone)]
pub struct CardPath(SmallVec<[Tag; 16]>);

impl Deref for CardPath {
    type Target = SmallVec<[Tag; 16]>;
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
#[derive(Debug, Clone)]
pub struct Card {
    pub name: String,
    pub id: String,
    pub paths: Vec<CardPath>,
    /// Typst source for the question
    question: String,
    /// Typst source for the answer
    answer: String,
    /// Its own handle in the CardStore
    pub handle: CardHandle,
}

/// Array of the Ratings for ease of use (i.e for rating in RATINGS ...)
pub const RATINGS: [Rating; 4] = [Rating::Again, Rating::Hard, Rating::Good, Rating::Easy];

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
    /// Returns the score of the rating [0-3]
    pub fn score(&self) -> u32 {
        match self {
            Rating::Again => 0,
            Rating::Hard => 1,
            Rating::Good => 2,
            Rating::Easy => 3,
        }
    }
}

impl Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str())
    }
}

/// Global store for cards and tags
pub struct CardStore {
    tags_map: HashMap<String, Tag>,
    /// Name of tags
    pub tags: SlotMap<Tag, String>,
    /// Cards
    pub cards: SlotMap<CardHandle, Card>,
}

impl Default for CardStore {
    fn default() -> Self {
        Self {
            tags_map: HashMap::new(),
            tags: SlotMap::with_key(),
            cards: SlotMap::with_key(),
        }
    }
}

/// Config for things that the source needs to compile
#[derive(Clone, Copy)]
pub struct SourceConfig {
    /// Page width in pt
    pub page_width: u32,
    // Text size in pt
    pub text_size: u32,
}

impl CardStore {
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

            let tag = self.tags.insert(str[index..].to_owned());
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
        let mut w = Vec::new();
        writeln!(&mut w, "#import \"cards_internal.typ\": *")?;
        writeln!(&mut w, "#show: setup")?;
        writeln!(&mut w, "#set page(width: {}pt)", config.page_width)?;
        writeln!(&mut w, "#set text(size: {}pt)", config.text_size)?;
        for card in cards {
            let card = &self.cards[card];
            write!(&mut w, "#card(\"{}\", \"{}\", (", card.id, card.name)?;
            for path in &card.paths {
                write!(&mut w, "\"")?;
                for chunk in
                    Itertools::intersperse(path.iter().map(|&t| self.tags[t].as_str()), ".")
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
        use nom::{
            bytes::*, character::*, combinator::*, error::*, multi::*, sequence::*, IResult, Parser,
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

        // Skip header
        let Ok((content, _)) = take_until::<&str, &str, Error<&str>>("#card").parse(content) else {
            // No card in file
            return Ok(());
        };

        // Parse cards
        let (_, (cards, _)) = many_till(card, eof).parse(content)?;

        // Resolve tags, names, handles
        for (id, name, paths, question, answer) in cards {
            let paths = paths
                .into_iter()
                .map(|s| CardPath::from_str(s, self))
                .collect();

            self.cards.insert_with_key(|k| Card {
                id: id.to_owned(),
                name: name.to_owned(),
                paths,
                question: question.to_owned(),
                answer: answer.to_owned(),
                handle: k,
            });
        }

        Ok(())
    }

    pub fn all_cards(&self) -> Vec<CardHandle> {
        self.cards.keys().collect()
    }
}
