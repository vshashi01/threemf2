//! Small helpers shared by the hand-written `FromXml` implementations.

use std::borrow::Cow;

use instant_xml::{Deserializer, Error, FromXml, de::Node};

/// Deserializes an attribute value into the accumulator of `T`, the same way the
/// `FromXml` derive does for `#[xml(attribute)]` fields.
pub(crate) fn read_attribute<'xml, T: FromXml<'xml>>(
    accumulator: &mut T::Accumulator,
    field: &'static str,
    deserializer: &mut Deserializer<'_, 'xml>,
    value: Cow<'xml, str>,
) -> Result<(), Error> {
    let mut nested = deserializer.for_node(Node::AttributeValue(value));
    T::deserialize(accumulator, field, &mut nested)
}
