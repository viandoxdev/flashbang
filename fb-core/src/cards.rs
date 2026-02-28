//! Contains logic related to cards:
//!  - parsing
//!  - splitting
//!  - building typst source files

use std::ops::Deref;
use std::sync::Arc;
use std::usize;

use itertools::Itertools;

use crate::Core;
use crate::error::CoreError;

#[derive(uniffi::Object)]
pub struct HeaderInfoInner {
    inner: String,
    id: u64,
}

uniffi::custom_newtype!(HeaderInfo, Arc<HeaderInfoInner>);

#[derive(Clone)]
pub struct HeaderInfo(Arc<HeaderInfoInner>);

#[derive(uniffi::Record)]
pub struct CardInfo {
    id: String,
    name: String,
    locations: Vec<String>,
    header: Option<HeaderInfo>,
    question: String,
    answer: String,
}

#[uniffi::export(with_foreign)]
pub trait CardSource: Send + Sync {
    fn header_content(&self) -> Option<String>;
    fn header_eq(&self, other: Option<Arc<dyn CardSource>>) -> bool;
    fn id(&self) -> String;
    fn name(&self) -> String;
    fn question(&self) -> String;
    fn answer(&self) -> String;
    fn locations(&self) -> Vec<String>;
}

impl<T: CardSource + ?Sized> CardSource for Arc<T> {
    fn header_content(&self) -> Option<String> {
        (**self).header_content()
    }
    fn header_eq(&self, other: Option<Arc<dyn CardSource>>) -> bool {
        (**self).header_eq(other)
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

impl HeaderInfo {
    pub fn new(content: &str, id: u64) -> Self {
        HeaderInfo(Arc::new(HeaderInfoInner {
            inner: content.to_owned(),
            id,
        }))
    }
}

#[uniffi::export]
impl HeaderInfoInner {
    fn content(&self) -> String {
        self.inner.clone()
    }
    fn id(&self) -> u64 {
        self.id
    }
}

impl Deref for HeaderInfo {
    type Target = HeaderInfoInner;
    fn deref(&self) -> &HeaderInfoInner {
        &self.0
    }
}

impl PartialEq for HeaderInfo {
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
    pub text_color: u32,
}

pub struct CardState {}

impl CardState {
    pub fn new() -> Self {
        Self {}
    }
}

pub trait CardCore {
    fn parse<'a>(&self, id: u64, content: &'a str) -> Result<Vec<CardInfo>, CoreError>;
    fn build_source(
        &self,
        cards: impl IntoIterator<Item = Arc<dyn CardSource>>,
        config: SourceConfig,
    ) -> Result<String, CoreError>;
}

impl CardCore for Core {
    /// Build the source for a set of cards and a config
    fn build_source(
        &self,
        cards: impl IntoIterator<Item = Arc<dyn CardSource>>,
        config: SourceConfig,
    ) -> Result<String, CoreError> {
        const CARDS_INTERNAL: &'static str = include_str!("./cards_internal.typ");

        use std::io::Write;

        let mut w = Vec::new();
        let mut last_card = None;

        writeln!(&mut w, "{CARDS_INTERNAL}")?;
        writeln!(&mut w, "#set page(width: {}pt)", config.page_width)?;
        writeln!(
            &mut w,
            "#let _colors = (text: rgb(\"#{:06X}\"))",
            config.text_color
        )?;
        writeln!(&mut w, "#let _sizes = (text: {}pt)", config.text_size)?;
        writeln!(&mut w, "#set text(size: _sizes.text, fill: _colors.text)")?;
        writeln!(&mut w, "#[")?;

        for card in cards {
            if !card.header_eq(last_card) {
                let current_header = card.header_content();
                writeln!(&mut w, "]")?;
                writeln!(&mut w, "#[")?;

                if let Some(header) = (&current_header).as_ref() {
                    writeln!(&mut w, "{}", header)?;
                }
            }

            write!(&mut w, "#card(\"{}\", \"{}\", (", card.id(), card.name())?;
            for full_path in card.locations() {
                write!(&mut w, "\"{full_path}\",")?;
            }
            writeln!(&mut w, "))")?;

            write!(&mut w, "{}", card.question())?;
            writeln!(&mut w, "#answer")?;
            write!(&mut w, "{}", card.answer())?;

            last_card = Some(card);
        }

        writeln!(&mut w, "]")?;

        Ok(String::from_utf8(w).map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::Other, "Couldn't parse bytes to string")
        })?)
    }

    /// Parse a typst source file for the cards inside
    fn parse<'a>(&self, id: u64, content: &'a str) -> Result<Vec<CardInfo>, CoreError> {
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
            match take_until::<&str, &str, Error<&str>>("//![FLASHBANG HEADER]").parse(content) {
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
            Some(HeaderInfo::new(header, id))
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
