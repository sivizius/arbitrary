//! Attributes to apply to one field in a `struct` or in an `enum` variant.

use {
    //self::options::{FieldOption, OptionName},
    super::OptionName,
    proc_macro2::TokenStream,
    quote::{quote, ToTokens},
    syn::Type,
};

/// Determines how a value for a field should be constructed.
#[derive(Clone, Default)]
#[cfg_attr(test, derive(Debug))]
pub enum FieldConstructor {
    /// Assume that Arbitrary is defined for the type of this field and use it (default)
    #[default]
    Arbitrary,

    /// Places `Default::default()` as a field value.
    Default,

    /// Use custom function or closure to generate a value for a field.
    With(TokenStream),

    /// Set a field always to the given value.
    Value(TokenStream),
}

impl FieldConstructor {
    /// Generate the constructor for these field attributes.
    pub(crate) fn generate_constructor(&self, u: impl ToTokens) -> TokenStream {
        match self {
            Self::Default => quote! { Default::default() },
            Self::Arbitrary => quote! { arbitrary::Arbitrary::arbitrary(#u)? },
            Self::With(function_or_closure) => quote! { (#function_or_closure)(#u)? },
            Self::Value(value) => quote! { #value },
        }
    }

    /// Generate the constructor for the final field for these attributes.
    pub(crate) fn generate_constructor_take_rest(&self) -> TokenStream {
        match self {
            Self::Default => quote! { Default::default() },
            Self::Arbitrary => quote! { arbitrary::Arbitrary::arbitrary_take_rest(u)? },
            Self::With(function_or_closure) => quote! { (#function_or_closure)(&mut u)? },
            Self::Value(value) => quote! { #value },
        }
    }

    pub(crate) fn generate_size_hint(&self, ty: &Type) -> TokenStream {
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
}

impl AsRef<str> for FieldConstructor {
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
