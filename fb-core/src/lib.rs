use android_logger::{Config, FilterBuilder};

uniffi::setup_scaffolding!();

mod cards;
mod compile;
mod github;

macro_rules! new_type_index {
    ($name:ident) => {
        uniffi::custom_newtype!($name, u64);

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub struct $name(u64);

        impl $name {
            fn index(self) -> usize {
                self.0 as usize
            }
        }

        impl From<usize> for $name {
            fn from(value: usize) -> Self {
                Self(value as u64)
            }
        }
    };
}

pub(crate) use new_type_index;

#[uniffi::export]
#[uniffi::method(name = "rustSetupLogger")]
pub fn rust_setup_logger() {
    android_logger::init_once(
        Config::default()
            .with_max_level(log::LevelFilter::Trace)
            .with_filter(FilterBuilder::new().parse("info,fb-core::crate=debug").build())
            .with_tag("fb-core"),
    );
}
