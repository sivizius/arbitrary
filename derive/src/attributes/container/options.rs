//! Inner attributes of container attributes.
//!
//! One container attribute (`#[arbitrary(…)]`) might consist of multiple options (`…`).

use {
    super::super::AttributeName,
    syn::{
        punctuated::Punctuated, Error, Expr, ExprLit, Ident, Lit, Meta, MetaNameValue, Token,
        TypeParam,
    },
};

/// A single parsed container option.
#[derive(Clone, Eq, Hash, PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub(crate) enum ContainerOption {
    /// Specify type bounds to be applied to the derived `Arbitrary` implementation
    ///   instead of the default inferred bounds.
    Bound(Punctuated<TypeParam, Token![,]>),
}

impl TryFrom<&Meta> for ContainerOption {
    type Error = Error;

    fn try_from(meta: &Meta) -> Result<Self, Self::Error> {
        let option_name = meta.try_into()?;

        match option_name {
            OptionName::Bound => match meta.require_name_value()? {
                MetaNameValue {
                    value:
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(bound_str_lit),
                            ..
                        }),
                    ..
                } => bound_str_lit
                    .parse_with(Punctuated::parse_terminated)
                    .map(Self::Bound),
                meta_name_value => Err(Error::new_spanned(
                    meta_name_value,
                    format!(
                        "Could not parse option `{}` of `#[{}]` container attribute: \
                        Expected `bound = \"$($param: $bounds),*\"`",
                        option_name.as_ref(),
                        AttributeName::Arbitrary.as_ref(),
                    ),
                )),
            },
        }
    }
}

/// Names of implemented options.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(test, derive(Debug))]
pub(crate) enum OptionName {
    /// Specify type bounds to be applied to the derived `Arbitrary` implementation
    ///   instead of the default inferred bounds.
    Bound,
}

impl OptionName {
    /// List of all known options as pairs of their name and their `enum`-variant.
    const ALL: [(&'static str, Self); 1] = [("bound", OptionName::Bound)];

    /// Return an array of all option names.
    ///
    /// Used e.g. to list all known options if unknown option was encountered.
    fn all_names() -> [&'static str; 1] {
        Self::ALL.map(|(name, _)| name)
    }

    /// Find attribute named `ident` or return `None` if unknown.
    ///
    /// This will perform a simple, linear search.
    fn find(ident: &Ident) -> Option<Self> {
        Self::ALL
            .iter()
            .find_map(|(name, option_name)| (ident == *name).then_some(*option_name))
    }
}

impl AsRef<str> for OptionName {
    /// Cheap `.to_string()`:
    fn as_ref(&self) -> &str {
        <&'static str>::from(*self)
    }
}

impl From<OptionName> for &'static str {
    /// Cheap `.to_string()`:
    fn from(option_name: OptionName) -> Self {
        match option_name {
            OptionName::Bound => OptionName::ALL[0].0,
        }
    }
}

impl TryFrom<&Ident> for OptionName {
    type Error = Error;

    /// Try to find `ident` in the list of known options:
    fn try_from(ident: &Ident) -> Result<Self, Self::Error> {
        Self::find(ident).ok_or_else(|| {
            Error::new(
                ident.span(),
                format!(
                    "Unknown option for `#[{}]`: `{ident}`. \
                    Known options for container attributes are: {:?}",
                    AttributeName::Arbitrary.as_ref(),
                    Self::all_names(),
                ),
            )
        })
    }
}

impl TryFrom<&Meta> for OptionName {
    type Error = Error;

    /// Parse a single `<option>`/`<option> = <value>`/`<option>(<inner>)`:
    fn try_from(meta: &Meta) -> Result<Self, Self::Error> {
        meta.path().require_ident()?.try_into()
    }
}
