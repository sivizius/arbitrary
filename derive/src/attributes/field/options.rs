//! Inner attributes of field attributes.
//!
//! One field attribute (`#[arbitrary(…)]`) might consist of multiple options (`…`).

use {
    super::super::ARBITRARY_ATTRIBUTE_NAME,
    syn::{spanned::Spanned, Error, Ident, Meta},
};

/// Names of implemented options.
#[derive(Clone, Copy)]
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
}

impl AsRef<str> for OptionName {
    /// Cheap `.to_string()`:
    fn as_ref(&self) -> &str {
        match self {
            Self::Default => Self::ALL[0].0,
            Self::With => Self::ALL[1].0,
            Self::Value => Self::ALL[2].0,
        }
    }
}

impl TryFrom<&Ident> for OptionName {
    type Error = String;

    /// Try to find `ident` in the list of known options:
    fn try_from(ident: &Ident) -> Result<Self, Self::Error> {
        Self::ALL
            .iter()
            .find_map(|(name, option_name)| (ident == *name).then_some(*option_name))
            .ok_or_else(|| {
                format!(
                    r#"Unknown option for #[{ARBITRARY_ATTRIBUTE_NAME}]: `{ident}`. Known options are: {:?}"#,
                    Self::ALL.map(|(name, _)| name),
                )
            })
    }
}

impl TryFrom<&Meta> for OptionName {
    type Error = Error;

    /// Parse a single `<option>`/`<option> = <value>`/`<option>(<inner>)`:
    fn try_from(meta: &Meta) -> Result<Self, Self::Error> {
        meta.path()
            .get_ident()
            .ok_or_else(|| {
                Error::new(
                    meta.span(),
                    format!("#[{ARBITRARY_ATTRIBUTE_NAME}] cannot be empty."),
                )
            })
            .and_then(|ident| {
                ident
                    .try_into()
                    .map_err(|message| Error::new(meta.span(), message))
            })
    }
}
