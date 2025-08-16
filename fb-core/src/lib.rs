use android_logger::{Config, FilterBuilder};

uniffi::setup_scaffolding!();

mod cards;
mod world;
mod github;
mod studies;
mod fsrs;

// We're trying to get close to the OOP model here (because it makes working with kotlin easier /
// cleaner).
//
// Objects are essentially all arcs, we have at least two structs for each:
//  - ObjectInner: immutable data holder
//  - Object: newtype around Arc<ObjectInner>
// And two optional ones:
//  - ObjectState: mutable state, ObjectInner will have a
//      state: Mutex<ObjectState>
//    field.
//  - ObjectWeak: Same as object but wraps a weak arc instead, not ffi safe, and provides the
//    following methods:
//      Object::downgrade(&self) -> ObjectWeak
//      ObjectWeak::upgrade(&self) -> Option<Object>
//
// This is very heavy (though verbosity is fine because of the macro), but fits the Kotlin OOP
// model nicely (no carrying handles around, no single source of truth). The downside is this is as
// far from idiomatic rust as we can get, and performs worse (although I doubt this is an issue).
//
// I am mostly experimenting for now, this could be refractored back into a Handle / Store
// architecture.
//
// NOTE: Not sure what to make of stores anymore, they are still necessary, but don't carry the
// same responsibilities as before: they currently just let me find tags and cards by id / path.

macro_rules! arc_struct {
    (@impl(inner) 
        vis: $v:vis,
        name: $name:ident,
        inner: $inner:ident,
        fields: {
            $($fields:tt)*
        },
        $(attr: $attr:meta,)?
        $(state: $state:ident,)?
    ) => {
        $(#[$attr])?
        #[derive(uniffi::Object)]
        $v struct $inner {
            $(
                state: ::parking_lot::Mutex<$state>,
            )?
            $($fields)*
        }

        $(#[$attr])?
        #[derive(Clone)]
        $v struct $name(::std::sync::Arc<$inner>);

        uniffi::custom_newtype!($name, ::std::sync::Arc<$inner>);

        impl ::std::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &*self.0
            }
        }

        impl ::std::cmp::PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                ::std::sync::Arc::ptr_eq(&self.0, &other.0)
            }
        }

        impl ::std::cmp::Eq for $name {}

        impl ::std::hash::Hash for $name {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                ::std::sync::Arc::as_ptr(&self.0).hash(state);
            }
        }
    };
    (@impl(state) 
        vis: $v:vis,
        name: $name:ident,
        fields: {
            $($fields:tt)*
        },
        $(attr: $attr:meta,)?
    ) => {
        $(#[$attr])?
        $v struct $name {
            $($fields)*
        }
    };
    (@impl(weak) 
        vis: $v:vis,
        name: $name:ident,
        inner: $inner:ident,
        weak: $weak:ident,
        $(attr: $attr:meta,)?
    ) => {
        impl $name {
            fn downgrade(&self) -> $weak {
                $weak(::std::sync::Arc::downgrade(&self.0))
            }
        }

        $(#[$attr])?
        #[derive(Clone)]
        $v struct $weak(::std::sync::Weak<$inner>);

        impl $weak {
            fn upgrade(&self) -> Option<$name> {
                Some($name(self.0.upgrade()?))
            }
        }

        impl ::std::cmp::PartialEq for $weak {
            fn eq(&self, other: &Self) -> bool {
                ::std::sync::Weak::ptr_eq(&self.0, &other.0)
            }
        }

        impl ::std::cmp::Eq for $weak {}

        impl ::std::hash::Hash for $weak {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                ::std::sync::Weak::as_ptr(&self.0).hash(state);
            }
        }
    };
    (
        $(
            $v:vis struct $name:ident {
                $(
                    $(#[$weak_attr:meta])?
                    weak $weak:ident;
                )?

                $(#[$inner_attr:meta])?
                inner $inner:ident {
                    $($inner_fields:tt)*
                }

                $(
                    $(#[$state_attr:meta])?
                    state $state:ident {
                        $($state_fields:tt)*
                    }           
                )?
            }
        )*
    ) => {
        $(
            $(
                arc_struct!(@impl(state)
                    vis: $v,
                    name: $state,
                    fields: {
                        $($state_fields)*
                    },
                    $(attr: $state_attr,)?
                );
            )?
            arc_struct!(@impl(inner) 
                vis: $v,
                name: $name,
                inner: $inner,
                fields: {
                    $($inner_fields)*
                },
                $(attr: $inner_attr,)?
                $(state: $state,)?
            );
            $(
                arc_struct!(@impl(weak)
                    vis: $v,
                    name: $name,
                    inner: $inner,
                    weak: $weak,
                    $(attr: $weak_attr,)?
                );
            )?
        )+
    };
}

pub(crate) use arc_struct;

#[uniffi::export]
#[uniffi::method(name = "rustSetupLogger")]
pub fn rust_setup_logger() {
    android_logger::init_once(
        Config::default()
            .with_max_level(log::LevelFilter::Trace)
            .with_filter(FilterBuilder::new().parse("info,fb-core::crate=trace").build())
            .with_tag("fb-core"),
    );
}
