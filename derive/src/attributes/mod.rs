pub(crate) mod container;
pub(crate) mod field;
pub(crate) mod variant;

use syn::{Ident, Meta};

pub(crate) use self::{
    container::ContainerAttributes, field::FieldAttributes, variant::VariantAttributes,
};

/// Name of this derive-macro’s helper-attribute.
pub(crate) const ARBITRARY_ATTRIBUTE_NAME: &str = "arbitrary";

/// Names of attributes to parse.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(test, derive(Debug))]
pub(crate) enum AttributeName {
    /// `#[arbitrary(…)]` attribute.
    Arbitrary,
}

impl AttributeName {
    /// List of all known attributes as pairs of their name and their `enum`-variant.
    ///
    /// TODO: Parse [`validator`]``
    ///
    /// [`validator`]: https://docs.rs/validator/latest/validator/
    const ALL: [(&'static str, Self); 1] = [("arbitrary", Self::Arbitrary)];

    /// Find attribute named `ident` or return `None` if unknown.
    ///
    /// This will perform a simple, linear search.
    fn find(ident: &Ident) -> Option<Self> {
        Self::ALL
            .iter()
            .find_map(|(name, attr_name)| (ident == *name).then_some(*attr_name))
    }

    /// Get path of `meta`, look for ident
    ///   and try to find attribute name in list of known attribute names.
    ///
    /// If any step failed, this will return `None`.
    ///
    /// This will perform a simple, linear search.
    fn find_meta(meta: &Meta) -> Option<Self> {
        meta.path().get_ident().and_then(Self::find)
    }
}

impl AsRef<str> for AttributeName {
    /// Cheap `.to_string()`:
    fn as_ref(&self) -> &str {
        match self {
            Self::Arbitrary => Self::ALL[0].0,
        }
    }
}

impl From<AttributeName> for &'static str {
    /// Cheap `.to_string()`:
    fn from(attr_name) -> &str {
        match self {
            Self::Arbitrary => Self::ALL[0].0,
        }
    }
}
