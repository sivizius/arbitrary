//! Attributes to apply to one field in a `struct` or in an `enum` variant.

mod options;

use {
    super::ARBITRARY_ATTRIBUTE_NAME,
    proc_macro2::TokenStream,
    quote::{quote, ToTokens},
    syn::{
        punctuated::Punctuated, spanned::Spanned, Attribute, Error, Field, Meta, MetaNameValue,
        Token, Type,
    },
};

use self::options::OptionName;

/// Determines how a value for a field should be constructed.
#[cfg_attr(test, derive(Debug))]
pub enum FieldAttributes {
    /// Assume that Arbitrary is defined for the type of this field and use it (default)
    Arbitrary,

    /// Places `Default::default()` as a field value.
    Default,

    /// Use custom function or closure to generate a value for a field.
    With(TokenStream),

    /// Set a field always to the given value.
    Value(TokenStream),
}

impl FieldAttributes {
    /// Generate the constructor for these field attributes.
    pub(crate) fn generate_constructor(self, u: impl ToTokens) -> TokenStream {
        match self {
            Self::Default => quote! { Default::default() },
            Self::Arbitrary => quote! { arbitrary::Arbitrary::arbitrary(#u)? },
            Self::With(function_or_closure) => quote! { (#function_or_closure)(#u)? },
            Self::Value(value) => quote! { #value },
        }
    }
    /// Generate the constructor for the final field for these attributes.
    pub(crate) fn generate_constructor_take_rest(self) -> TokenStream {
        match self {
            Self::Default => quote! { Default::default() },
            Self::Arbitrary => quote! { arbitrary::Arbitrary::arbitrary_take_rest(u)? },
            Self::With(function_or_closure) => quote! { (#function_or_closure)(&mut u)? },
            Self::Value(value) => quote! { #value },
        }
    }

    pub(crate) fn generate_size_hint(self, ty: &Type) -> TokenStream {
        match self {
            Self::Default | Self::Value(_) => quote! { (0, Some(0)) },
            Self::Arbitrary => quote! { <#ty as arbitrary::Arbitrary>::size_hint(depth) },

            // NOTE: In this case it's hard to determine what size_hint must be,
            //   so size_of::<T>() is just an educated guess,
            //   although it's gonna be inaccurate for dynamically
            //   allocated types (Vec, HashMap, etc.).
            Self::With(_) => quote! { (::core::mem::size_of::<#ty>(), None) },
        }
    }

    /// Parse the next option and return a new state.
    fn next(self, meta: Meta) -> Result<Self, Error> {
        match (self, (&meta).try_into()?) {
            (Self::Arbitrary, OptionName::Default) => {
                let _path = meta.require_path_only()?;
                Ok(Self::Default)
            }
            (Self::Arbitrary, OptionName::With) => {
                let MetaNameValue { value, .. } = meta.require_name_value()?;
                Ok(Self::With(value.to_token_stream()))
            }
            (Self::Arbitrary, OptionName::Value) => {
                let MetaNameValue { value, .. } = meta.require_name_value()?;
                Ok(Self::Value(value.to_token_stream()))
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
}

impl AsRef<str> for FieldAttributes {
    /// Cheap `.to_string()`:
    fn as_ref(&self) -> &str {
        match self {
            Self::Arbitrary => "",
            Self::Default => OptionName::Default.as_ref(),
            Self::With(_) => OptionName::With.as_ref(),
            Self::Value(_) => OptionName::Value.as_ref(),
        }
    }
}

impl TryFrom<&Attribute> for FieldAttributes {
    type Error = Error;

    /// Parse the `…` in `#[arbitrary(…)]`:
    fn try_from(attr: &Attribute) -> Result<Self, Self::Error> {
        attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .map_err(|error| {
                Error::new(
                    attr.span(),
                    format!("#[{ARBITRARY_ATTRIBUTE_NAME}] must contain a group: {error}"),
                )
            })?
            .into_iter()
            .try_fold(Self::Arbitrary, Self::next)
    }
}

impl TryFrom<&Field> for FieldAttributes {
    type Error = Error;

    /// Parse the attributes of a field:
    fn try_from(field: &Field) -> Result<Self, Self::Error> {
        // Filter out all non-`abitrary`-attributes for now.
        // TODO: We might want to check e.g. `validate`, `schemars`, …
        let mut attrs = field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident(ARBITRARY_ATTRIBUTE_NAME));

        match (attrs.next(), attrs.next()) {
            // No `arbitrary`-attributes: Ok
            (None, _) => Ok(Self::Arbitrary),
            // One `arbitrary`-attributes: Ok
            (Some(attr), None) => attr.try_into(),
            // Multiple `arbitrary`-attributes: Error
            // TODO: We might want to try to combine them.
            _ => Err(Error::new(
                field.span(),
                format!(
                    "Multiple conflicting #[{ARBITRARY_ATTRIBUTE_NAME}] attributes found on field `{}`",
                    field.ident.as_ref().unwrap()
                ),
            )),
        }
    }
}
