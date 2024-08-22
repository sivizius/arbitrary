//! Attributes to apply to  in a `struct` or in an `enum` variant.

mod options;

use {
    self::options::ContainerOption,
    super::AttributeName,
    syn::{parse::Error, punctuated::Punctuated, Attribute, DeriveInput, Meta, Token, TypeParam},
};

/// Cumulated attributes for a container.
#[derive(Clone, Default, Eq, Hash, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub struct ContainerAttributes {
    /// Specify type bounds to be applied to the derived `Arbitrary` implementation instead of the
    /// default inferred bounds.
    ///
    /// ```ignore
    /// #[arbitrary(bound = "T: Default, U: Debug")]
    /// ```
    ///
    /// Multiple attributes will be combined as long as they don't conflict, e.g.
    ///
    /// ```ignore
    /// #[arbitrary(bound = "T: Default")]
    /// #[arbitrary(bound = "U: Default")]
    /// ```
    pub bounds: Option<Punctuated<TypeParam, Token![,]>>,
}

impl ContainerAttributes {
    /// Parse the next option and return a new state.
    fn next_option(mut self, meta: &Meta) -> Result<Self, Error> {
        match meta.try_into()? {
            ContainerOption::Bound(bounds) => {
                self.bounds.get_or_insert_with(<_>::default).extend(bounds);
                Ok(self)
            }
        }
    }

    /// Process next `#[…]` attribute.
    fn next_attribute(self, Attribute { meta, .. }: &Attribute) -> Result<Self, Error> {
        match AttributeName::find_meta(meta) {
            Some(AttributeName::Arbitrary) => meta
                .require_list()?
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
                .iter()
                .try_fold(self, Self::next_option),
            None => Ok(self),
        }
    }
}

impl TryFrom<&DeriveInput> for ContainerAttributes {
    type Error = Error;

    fn try_from(DeriveInput { attrs, .. }: &DeriveInput) -> Result<Self, Error> {
        attrs.iter().try_fold(Self::default(), Self::next_attribute)
    }
}
