pub mod cards;
pub mod error;
#[cfg(feature = "fuzzy")]
pub mod fuzzy;
#[cfg(feature = "compile")]
pub mod world;
#[cfg(feature = "compile")]
pub mod packages;
#[cfg(feature = "scheduler")]
pub mod scheduler;
#[cfg(feature = "github")]
mod github;
#[cfg(feature = "cache")]
pub mod cache;

#[cfg(feature = "compile")]
pub use typst;
#[cfg(feature = "scheduler")]
pub use fsrs;

#[cfg(feature = "uniffi")]
uniffi::setup_scaffolding!();
