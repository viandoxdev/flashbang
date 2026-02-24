#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum CoreError {
    #[error("couldn't parse typst file for cards: {details}")]
    Parsing { details: String },
    #[error("IO error: {details}")]
    IO { details: String },
    #[error("Http (Reqwest) error: {details}")]
    HTTP { details: String },
    #[error("Typst error: {details}")]
    Typst { details: String },
    #[error("FSRS error: {details}")]
    FSRS { details: String },
}

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for CoreError {
    fn from(value: nom::Err<nom::error::Error<&'a str>>) -> Self {
        Self::Parsing {
            details: match value {
                nom::Err::Incomplete(needed) => match needed {
                    nom::Needed::Unknown => format!("Incomplete input"),
                    nom::Needed::Size(size) => format!("Incomplete input, missing ({size}) bytes"),
                },
                nom::Err::Error(err) => format!("Error while parsing: {err} ({err:?})"),
                nom::Err::Failure(fail) => format!("Failed to parse: {fail} ({fail:?})"),
            },
        }
    }
}

impl From<std::io::Error> for CoreError {
    fn from(value: std::io::Error) -> Self {
        Self::IO {
            details: format!("{value} ({value:?})"),
        }
    }
}

impl From<reqwest::Error> for CoreError {
    fn from(value: reqwest::Error) -> Self {
        Self::HTTP {
            details: format!("{value} ({value:?})"),
        }
    }
}

impl From<fsrs::FSRSError> for CoreError {
    fn from(value: fsrs::FSRSError) -> Self {
        Self::FSRS {
            details: format!("{value} ({value:?})"),
        }
    }
}

