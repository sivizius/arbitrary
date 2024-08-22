//! FIXME: Missing Documentation.

use {
    crate::ARBITRARY_ATTRIBUTE_NAME,
    syn::{Meta, Variant},
};

/// FIXME: Missing Documentation.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariantAttributes;

impl VariantAttributes {
    /// FIXME: Missing Documentation.
    pub fn not_skipped(variant: &&Variant) -> bool {
        !should_skip(variant)
    }

    /// FIXME: Missing Documentation.
    fn should_skip(Variant { attrs, .. }: &Variant) -> bool {
        attrs
            .iter()
            .filter_map(|attr| {
                attr.path()
                    .is_ident(ARBITRARY_ATTRIBUTE_NAME)
                    .then(|| attr.parse_args::<Meta>())
                    .and_then(Result::ok)
            })
            .any(|meta| match meta {
                Meta::Path(path) => path.is_ident("skip"),
                _ => false,
            })
    }
}
