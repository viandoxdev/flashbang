pub mod cards;
pub mod error;
pub mod fuzzy;
pub mod world;
pub mod packages;
#[cfg(feature = "scheduler")]
pub mod scheduler;
#[cfg(feature = "github")]
mod github;
#[cfg(feature = "cache")]
pub mod cache;

pub use typst;
#[cfg(feature = "scheduler")]
pub use fsrs;

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
