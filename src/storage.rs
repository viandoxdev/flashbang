//! Common tools for disk storage, essentially nop on web, supports linux and android.

use std::{
    io::Write,
    ops::{Deref, DerefMut},
    path::{Path, PathBuf},
};

/// Wrapper around a Serializable type that allows reading from file on init
/// and saving to disk.
pub struct Storable<T> {
    value: T,
    path: Option<PathBuf>,
}

fn try_deserialize<T: serde::de::DeserializeOwned>(s: &str) -> Option<T> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        let n1 = c.to_digit(16)?;
        let c2 = chars.next()?;
        let n2 = c2.to_digit(16)?;
        bytes.push((n1 * 16 + n2) as u8);
    }
    postcard::from_bytes(&yazi::decompress(&bytes, yazi::Format::Zlib).ok()?.0).ok()
}

/// Types that can deserialize into another type used for backwards compatibility
trait FallbackDeserialize<T> {
    fn deserialize(s: &str) -> Option<T>;
}

/// Wrapper type that doesn't implement Deserialize to avoid conflicting implementation from using
/// raw tuples, unfortunate but needed.
///
/// Use as you would a tuple for lists of types
/// ```
/// (A, B, C) -> TypeList<(A, B, C)>
/// (A,) -> TypeList<(A,)>
/// () -> TypeList<()>
/// ```
pub struct TypeList<T>(T);

impl<T, F: Into<T> + serde::de::DeserializeOwned> FallbackDeserialize<T> for F {
    fn deserialize(s: &str) -> Option<T> {
        try_deserialize::<Self>(s).map(Self::into)
    }
}

macro_rules! impl_tuples {
    ($($t:tt)*) => {
        impl<T, $($t: FallbackDeserialize<T>,)*> FallbackDeserialize<T> for TypeList<($($t,)*)> {
            #[allow(unused_variables)]
            fn deserialize(s: &str) -> Option<T> {
                None
                    $(.or_else(|| $t::deserialize(s)))*
            }
        }
    };
}

impl_tuples!();
impl_tuples!(A);
impl_tuples!(A B);
impl_tuples!(A B C);
impl_tuples!(A B C D);
impl_tuples!(A B C D E);
impl_tuples!(A B C D E F);
impl_tuples!(A B C D E F G);
impl_tuples!(A B C D E F G H);
impl_tuples!(A B C D E F G H I);
impl_tuples!(A B C D E F G H I J);

/// Types that can be serialized and deserialized with potential fallback types (for versioning)
pub trait Serializable: serde::Serialize + serde::de::DeserializeOwned {
    /// TypeList of types that implement serde::de::DeserializeOwned and Into<Self>
    /// If this type can't be deserialized each of the fallback types will be tried
    #[allow(private_bounds)]
    type Fallback: FallbackDeserialize<Self>;

    fn deserialize(s: &str) -> Option<Self> {
        try_deserialize(s).or_else(|| Self::Fallback::deserialize(s))
    }

    fn serialize(&self) -> Option<String> {
        let serialized = postcard::to_allocvec(self).unwrap();
        let compressed = yazi::compress(
            &serialized,
            yazi::Format::Zlib,
            yazi::CompressionLevel::BestSize,
        )
        .ok()?;
        let as_str: String = compressed
            .iter()
            .flat_map(|u| {
                [
                    char::from_digit(((*u & 0xF0) >> 4).into(), 16).unwrap(),
                    char::from_digit((*u & 0x0F).into(), 16).unwrap(),
                ]
                .into_iter()
            })
            .collect();
        Some(as_str)
    }
}

impl<T: Serializable> Storable<T> {
    fn get(path: impl AsRef<Path>) -> Option<T> {
        let s = std::fs::read_to_string(path).ok()?;
        Serializable::deserialize(&s)
    }

    fn set(path: impl AsRef<Path>, v: &T) -> Option<()> {
        let mut file = std::fs::File::create(path).ok()?;
        let str = Serializable::serialize(v)?;
        file.write_all(str.as_bytes()).ok()
    }

    pub fn new(key: impl AsRef<str>, init: impl Fn() -> T) -> Self {
        let key = key.as_ref();
        #[cfg(target_os = "android")]
        let base_path = Some(PathBuf::from("/data/user/0/dev.vndx.Flashbang/files"));
        #[cfg(target_os = "linux")]
        let base_path = Some({
            directories::BaseDirs::new()
                .unwrap()
                .data_local_dir()
                .join(env!("CARGO_PKG_NAME"))
        });
        #[cfg(not(any(target_os = "android", target_os = "linux")))]
        let base_path: Option<PathBuf> = None;

        let (value, path) = if let Some(base_path) = base_path {
            std::fs::create_dir_all(&base_path).expect("Couldn't create directories for storage");

            let path = base_path.join(key);
            if let Some(val) = Storable::<T>::get(&path) {
                (val, Some(path))
            } else {
                let val = init();
                Storable::<T>::set(&path, &val);
                (val, Some(path))
            }
        } else {
            (init(), None)
        };

        Self { value, path }
    }

    pub fn save(&self) {
        if let Some(path) = &self.path {
            Storable::<T>::set(path, &self.value);
        }
    }
}

impl<T> Deref for Storable<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> DerefMut for Storable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}
