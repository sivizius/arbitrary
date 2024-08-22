//! Inner attributes of field attributes.
//!
//! One field attribute (`#[arbitrary(…)]`) might consist of multiple options (`…`).

use {
    super::super::AttributeName,
    proc_macro2::TokenStream,
    quote::ToTokens,
    syn::{Error, Ident, Meta, MetaNameValue},
};

/// A single parsed field option.
#[derive(Clone)]
#[cfg_attr(test, derive(Debug))]
pub enum FieldOption {
    /// Places `Default::default()` as a field value.
    Default,

    /// Use custom function or closure to generate a value for a field.
    With(TokenStream),

    /// Set a field always to the given value.
    Value(TokenStream),
}

impl AsRef<str> for FieldOption {
    /// Cheap `.to_string()`:
    fn as_ref(&self) -> &str {
        <&'static str>::from(self)
    }
}

impl From<&FieldOption> for &'static str {
    /// Cheap `.to_string()`:
    fn from(field_option: &FieldOption) -> Self {
        OptionName::from(field_option).into()
    }
}

impl TryFrom<&Meta> for FieldOption {
    type Error = Error;

    fn try_from(meta: &Meta) -> Result<Self, Self::Error> {
        let option_name = meta.try_into()?;

        match option_name {
            OptionName::Default => {
                let _path = meta.require_path_only()?;
                Ok(Self::Default)
            }
            OptionName::With => {
                let MetaNameValue { value, .. } = meta.require_name_value()?;
                Ok(Self::With(value.to_token_stream()))
            }
            OptionName::Value => {
                let MetaNameValue { value, .. } = meta.require_name_value()?;
                Ok(Self::Value(value.to_token_stream()))
            }
        }
    }
}

/// Names of implemented options.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(test, derive(Debug))]
pub(crate) enum OptionName {
    /// Places `Default::default()` as a field value.
    Default,

    /// Use custom function or closure to generate a value for a field.
    With,

    /// Set a field always to the given value.
    Value,
}

impl OptionName {
    /// List of all known options as pairs of their name and their `enum`-variant.
    const ALL: [(&'static str, Self); 3] = [
        ("default", OptionName::Default),
        ("with", OptionName::With),
        ("value", OptionName::Value),
    ];

    /// Return an array of all option names.
    ///
    /// Used e.g. to list all known options if unknown option was encountered.
    fn all_names() -> [&'static str; 3] {
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
            OptionName::Default => OptionName::ALL[0].0,
            OptionName::With => OptionName::ALL[1].0,
            OptionName::Value => OptionName::ALL[2].0,
        }
    }
}

impl From<&FieldOption> for OptionName {
    /// Cheap `.to_string()`:
    fn from(option: &FieldOption) -> OptionName {
        match option {
            FieldOption::Default => Self::Default,
            FieldOption::With(_) => Self::With,
            FieldOption::Value(_) => Self::Value,
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
                    Known options for field attributes are: {:?}",
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
