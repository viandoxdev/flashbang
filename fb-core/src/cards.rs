//! Contains logic related to cards:
//!  - parsing
//!  - splitting
//!  - building typst source files

use std::ops::Deref;
use std::sync::Arc;
use std::usize;

use itertools::Itertools;

use crate::error::CoreError;
use crate::Core;

#[derive(uniffi::Object)]
pub struct HeaderInner {
    inner: String,
}

uniffi::custom_newtype!(Header, Arc<HeaderInner>);

#[derive(Clone)]
pub struct Header(Arc<HeaderInner>);

#[derive(uniffi::Record)]
pub struct CardInfo {
    id: String,
    name: String,
    locations: Vec<String>,
    header: Option<Header>,
    question: String,
    answer: String,
}

#[uniffi::export(with_foreign)]
pub trait CardSource: Send + Sync {
    fn header(&self) -> Option<Header>;
    fn id(&self) -> String;
    fn name(&self) -> String;
    fn question(&self) -> String;
    fn answer(&self) -> String;
    fn locations(&self) -> Vec<String>;
}

impl<T: CardSource + ?Sized> CardSource for Arc<T> {
    fn header(&self) -> Option<Header> {
        (**self).header()
    }
    fn id(&self) -> String {
        (**self).id()
    }
    fn name(&self) -> String {
        (**self).name()
    }
    fn question(&self) -> String {
        (**self).question()
    }
    fn answer(&self) -> String {
        (**self).answer()
    }
    fn locations(&self) -> Vec<String> {
        (**self).locations()
    }
}

impl Header {
    pub fn new(content: &str) -> Self {
        Header(Arc::new(HeaderInner {
            inner: content.to_owned(),
        }))
    }
}

impl Deref for Header {
    type Target = HeaderInner;
    fn deref(&self) -> &HeaderInner {
        &self.0
    }
}

impl PartialEq for Header {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
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

pub struct CardState {}

impl CardState {
    pub fn new() -> Self {
        Self {}
    }
}

pub trait CardCore {
    fn parse<'a>(&self, content: &'a str) -> Result<Vec<CardInfo>, CoreError>;
    fn build_source<C: CardSource>(
        &self,
        cards: impl IntoIterator<Item = C>,
        config: SourceConfig,
    ) -> Result<String, CoreError>;
}

impl CardCore for Core {
    /// Build the source for a set of cards and a config
    fn build_source<C: CardSource>(
        &self,
        cards: impl IntoIterator<Item = C>,
        config: SourceConfig,
    ) -> Result<String, CoreError> {
        const CARDS_INTERNAL: &'static str = include_str!("./cards_internal.typ");

        use std::io::Write;

        let mut w = Vec::new();
        let mut last_header = None;

        writeln!(&mut w, "{CARDS_INTERNAL}")?;
        writeln!(&mut w, "#set page(width: {}pt)", config.page_width)?;
        writeln!(&mut w, "#set text(size: {}pt)", config.text_size)?;
        writeln!(&mut w, "#[")?;

        for card in cards {
            let current_header = card.header();
            if last_header != current_header {
                writeln!(&mut w, "]")?;
                writeln!(&mut w, "#[")?;

                if let Some(header) = (&current_header).as_ref() {
                    writeln!(&mut w, "{}", header.inner)?;
                }

                last_header = current_header;
            }

            write!(&mut w, "#card(\"{}\", \"{}\", (", card.id(), card.name())?;
            for full_path in card.locations() {
                write!(&mut w, "\"{full_path}\",")?;
            }
            writeln!(&mut w, "))")?;

            write!(&mut w, "{}", card.question())?;
            writeln!(&mut w, "#answer")?;
            write!(&mut w, "{}", card.answer())?;
        }

        writeln!(&mut w, "]")?;

        Ok(String::from_utf8(w).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "Couldn't parse bytes to string")
        })?)
    }

    /// Parse a typst source file for the cards inside
    fn parse<'a>(&self, content: &'a str) -> Result<Vec<CardInfo>, CoreError> {
        if content.starts_with("//![FLASHBANG IGNORE]")
            || content.starts_with("//![FLASHBANG INCLUDE]")
        {
            return Ok(Vec::new());
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
            return Ok(Vec::new());
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
        Ok(cards
            .into_iter()
            .map(|(id, name, locations, question, answer)| CardInfo {
                id: id.to_owned(),
                name: name.to_owned(),
                locations: locations.into_iter().map(ToOwned::to_owned).collect_vec(),
                header: header.clone(),
                question: question.to_owned(),
                answer: answer.to_owned(),
            })
            .collect_vec())
    }
}

// impl Tag {
//     fn new(full_path: &str, store: &mut CardStore) -> Self {
//         // Find the index of the last '.' character
//         // The tag's name is str[index..], the rest is there to make sure we can
//         // distinguish between to tags of the same name under different paths :
//         //  Maths.Algebra.Theorems != Maths.Calculus.Theorems
//         let index = full_path
//             .char_indices()
//             .rev()
//             .find_map(|(i, c)| (c == '.').then_some(i + 1))
//             .unwrap_or(0);
//         let inner = TagInner {
//             name: full_path[index..].to_owned(),
//             full_path: full_path.to_owned(),
//             state: Mutex::new(TagState {
//                 children: HashSet::new(),
//                 cards: HashSet::new(),
//                 indirect_cards: HashSet::new(),
//             }),
//         };
//
//         let tag = Tag(Arc::new(inner));
//
//         if let Some(parent) = tag.parent(store) {
//             parent.add_child(tag.clone());
//         }
//
//         let root = tag.root(store);
//
//         store.roots.insert(root);
//
//         tag
//     }
// }
