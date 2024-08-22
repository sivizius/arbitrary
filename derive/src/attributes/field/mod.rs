//! Attributes to apply to one field in a `struct` or in an `enum` variant.

mod constructor;
mod options;

use {
    self::{
        constructor::FieldConstructor,
        options::{FieldOption, OptionName},
    },
    super::AttributeName,
    proc_macro2::TokenStream,
    quote::{quote, ToTokens},
    syn::{punctuated::Punctuated, spanned::Spanned, Attribute, Error, Field, Meta, Token},
};

/// Cumulated attributes for a field.
#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub struct FieldAttributes<'f> {
    /// The field to which these attributes are applied to.
    field: &'f Field,

    /// Determines how a value for a field should be constructed.
    constructor: FieldConstructor,
}

impl<'f> FieldAttributes<'f> {
    /// Construct a new object representing the attributes of a field.
    fn new(field: &'f Field) -> Self {
        Self {
            field,
            constructor: <_>::default(),
        }
    }

    /// Generate the constructor for these field attributes.
    pub(crate) fn generate_constructor(&self, u: impl ToTokens) -> TokenStream {
        let Self {
            field: Field {
                ident, colon_token, ..
            },
            constructor,
        } = self;
        let value = constructor.generate_constructor(u);
        quote! { #ident #colon_token #value }
    }
    /// Generate the constructor for the final field for these attributes.
    pub(crate) fn generate_constructor_take_rest(&self) -> TokenStream {
        let Self {
            field: Field {
                ident, colon_token, ..
            },
            constructor,
        } = self;
        let value = constructor.generate_constructor_take_rest();
        quote! { #ident #colon_token #value }
    }

    pub(crate) fn generate_size_hint(&self) -> TokenStream {
        self.constructor.generate_size_hint(&self.field.ty)
    }

    /// Parse the next option and return a new state.
    fn next_option(mut self, meta: &Meta) -> Result<Self, Error> {
        match (self.constructor, meta.try_into()?) {
            (FieldConstructor::Arbitrary, FieldOption::Default) => {
                let _path = meta.require_path_only()?;
                self.constructor = FieldConstructor::Default;
                Ok(self)
            }
            (FieldConstructor::Arbitrary, FieldOption::With(tokens)) => {
                self.constructor = FieldConstructor::With(tokens);
                Ok(self)
            }
            (FieldConstructor::Arbitrary, FieldOption::Value(tokens)) => {
                self.constructor = FieldConstructor::Value(tokens);
                Ok(self)
            }
            (previous, current) => {
                let previous = previous.as_ref();
                let current = current.as_ref();
                Err(Error::new(
                    meta.span(),
                    format!("Option {previous} does not allow additional options, got {current}"),
                ))
            }
        }
    }

    /// Process next `#[…]` attribute.
    fn next_attribute(self, Attribute { meta, .. }: &Attribute) -> Result<Self, Error> {
        match AttributeName::find_meta(meta) {
            // Process `#[arbitrary(…)]`:
            Some(AttributeName::Arbitrary) => meta
                .require_list()?
                .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?
                .iter()
                .try_fold(self, Self::next_option),

            // Ignore any other attribute:
            None => Ok(self),
        }
    }
}

impl<'f> TryFrom<&'f Field> for FieldAttributes<'f> {
    type Error = Error;

    /// Parse the attributes of a field:
    fn try_from(field: &'f Field) -> Result<Self, Self::Error> {
        field
            .attrs
            .iter()
            .try_fold(Self::new(field), Self::next_attribute)
    }
}
