mod container;
mod field;

pub(crate) use self::{container::ContainerAttributes, field::FieldAttributes};

/// Name of this derive-macro’s helper-attribute:
pub(crate) const ARBITRARY_ATTRIBUTE_NAME: &str = "arbitrary";
