#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum CoreError {
    #[error("couldn't parse typst file for cards: {message}")]
    Parsing { message: String },
    #[error("IO error: {message}")]
    IO { message: String },
    #[error("Http (Reqwest) error: {message}")]
    HTTP { message: String },
    #[error("Typst error: {message}")]
    Typst { message: String },
    #[error("FSRS error: {message}")]
    FSRS { message: String },
}

impl<'a> From<nom::Err<nom::error::Error<&'a str>>> for CoreError {
    fn from(value: nom::Err<nom::error::Error<&'a str>>) -> Self {
        Self::Parsing {
            message: match value {
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
            message: format!("{value} ({value:?})"),
        }
    }
}

impl From<reqwest::Error> for CoreError {
    fn from(value: reqwest::Error) -> Self {
        Self::HTTP {
            message: format!("{value} ({value:?})"),
        }
    }
}

impl From<fsrs::FSRSError> for CoreError {
    fn from(value: fsrs::FSRSError) -> Self {
        Self::FSRS {
            message: format!("{value} ({value:?})"),
        }
    }
}

