pub(crate) mod container;
pub(crate) mod field;
pub(crate) mod variant;

pub(crate) use self::{
    container::ContainerAttributes, field::FieldAttributes, variant::VariantAttributes,
};

/// Name of this derive-macro’s helper-attribute.
pub(crate) const ARBITRARY_ATTRIBUTE_NAME: &str = "arbitrary";
