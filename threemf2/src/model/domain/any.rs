//! Support for the `xs:any` and `xs:anyAttribute` extension points of the 3MF schemas.
//!
//! The 3MF core schema allows foreign content (`namespace="##other"`, `processContents="lax"`)
//! on most of its elements. This module provides owned types to keep that content so it can be
//! inspected, created and round-tripped:
//!
//! - [`AnyAttribute`] / [`AnyAttributes`] for `xs:anyAttribute` slots.
//! - [`AnyElement`] / [`AnyNode`] for `xs:any` slots, including the full subtree of the element.
//!
//! Names are identified by their namespace URI and local name. Prefixes are only kept as hints
//! and are re-assigned when writing if they clash with prefixes already in use.
//!
//! Everything in this module is only available with the `any` feature.

use crate::{model::StrResource, threemf_namespaces::CORE_NS};

pub use crate::model::domain::any_extensions::AnyError;

/// Namespace URI bound to the reserved `xml` prefix.
pub const XML_NS: &str = "http://www.w3.org/XML/1998/namespace";

/// Namespace URI bound to the reserved `xmlns` prefix.
pub const XMLNS_NS: &str = "http://www.w3.org/2000/xmlns/";

/// Returns `true` if `ns` may be used for top-level content of an `##other` slot.
///
/// That is any namespace except "no namespace", the 3MF core namespace and the reserved
/// `xmlns` namespace.
pub fn is_foreign_namespace(ns: &str) -> bool {
    !ns.is_empty() && ns != CORE_NS && ns != XMLNS_NS
}

/// Returns `true` if `ns` may be used for `##other` content of an element in namespace `own`.
///
/// `##other` excludes the namespace of the schema that declares the slot, so foreign content of
/// an extension element must not use that extension's namespace either.
#[cfg(any(feature = "read", feature = "write"))]
pub(crate) fn is_foreign_for(ns: &str, own: &str) -> bool {
    is_foreign_namespace(ns) && ns != own
}

fn check_foreign(ns: &str) -> Result<(), AnyError> {
    match is_foreign_namespace(ns) {
        true => Ok(()),
        false => Err(AnyError::NotForeignNamespace(ns.to_owned())),
    }
}

/// Light-weight NCName check: non-empty, no `:`, no whitespace, and a valid first character.
fn is_ncname(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first.is_alphabetic() || first == '_') {
        return false;
    }

    chars.all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '\u{B7}'))
}

fn check_name(name: &str) -> Result<(), AnyError> {
    match is_ncname(name) {
        true => Ok(()),
        false => Err(AnyError::InvalidName(name.to_owned())),
    }
}

/// An XML attribute identified by namespace URI and local name.
///
/// An empty namespace means the attribute has no namespace. That is only valid for attributes
/// nested inside an [`AnyElement`]; attributes stored in [`AnyAttributes`] always have a foreign
/// namespace.
/// Equality compares namespace, name and value; the prefix hint is ignored because prefixes carry
/// no meaning in XML.
#[derive(Debug, Clone)]
pub struct AnyAttribute {
    ns: StrResource,
    name: StrResource,
    value: StrResource,
    prefix_hint: Option<StrResource>,
}

impl AnyAttribute {
    /// Creates a new attribute. `ns` may be empty for an attribute without namespace.
    pub fn new(ns: &str, name: &str, value: impl Into<StrResource>) -> Result<Self, AnyError> {
        check_name(name)?;
        Ok(Self {
            ns: ns.into(),
            name: name.into(),
            value: value.into(),
            prefix_hint: None,
        })
    }

    /// Sets the preferred prefix used when writing this attribute.
    pub fn with_prefix_hint(mut self, prefix: &str) -> Self {
        self.prefix_hint = is_ncname(prefix).then(|| prefix.into());
        self
    }

    /// Namespace URI of the attribute, empty if it has no namespace.
    pub fn ns(&self) -> &str {
        &self.ns
    }

    /// Local name of the attribute.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Decoded value of the attribute.
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Replaces the value of the attribute.
    pub fn set_value(&mut self, value: impl Into<StrResource>) {
        self.value = value.into();
    }

    /// Prefix seen when reading, or set with [`AnyAttribute::with_prefix_hint`].
    pub fn prefix_hint(&self) -> Option<&str> {
        self.prefix_hint.as_deref()
    }

    fn is(&self, ns: &str, name: &str) -> bool {
        &*self.ns == ns && &*self.name == name
    }
}

impl PartialEq for AnyAttribute {
    fn eq(&self, other: &Self) -> bool {
        self.ns == other.ns && self.name == other.name && self.value == other.value
    }
}

impl Eq for AnyAttribute {}

/// The foreign attributes of an element with an `xs:anyAttribute` slot.
///
/// All attributes have a foreign namespace (see [`is_foreign_namespace`]) and each
/// `(namespace, name)` pair appears at most once.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnyAttributes(Vec<AnyAttribute>);

impl AnyAttributes {
    /// A shared empty collection, for slots that cannot hold attributes.
    pub(crate) fn empty() -> &'static Self {
        static EMPTY: AnyAttributes = AnyAttributes(Vec::new());
        &EMPTY
    }

    /// Creates an empty collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the value of the attribute `name` in namespace `ns`, if present.
    pub fn get(&self, ns: &str, name: &str) -> Option<&str> {
        self.get_attribute(ns, name).map(AnyAttribute::value)
    }

    /// Returns the attribute `name` in namespace `ns`, if present.
    pub fn get_attribute(&self, ns: &str, name: &str) -> Option<&AnyAttribute> {
        self.0.iter().find(|a| a.is(ns, name))
    }

    /// Sets the attribute `name` in namespace `ns`, replacing an existing value.
    pub fn set(
        &mut self,
        ns: &str,
        name: &str,
        value: impl Into<StrResource>,
    ) -> Result<(), AnyError> {
        self.insert(AnyAttribute::new(ns, name, value)?)
    }

    /// Inserts an attribute, replacing an existing attribute with the same name.
    pub fn insert(&mut self, attribute: AnyAttribute) -> Result<(), AnyError> {
        check_foreign(attribute.ns())?;
        match self
            .0
            .iter_mut()
            .find(|a| a.is(attribute.ns(), attribute.name()))
        {
            Some(existing) => *existing = attribute,
            None => self.0.push(attribute),
        }
        Ok(())
    }

    /// Removes and returns the attribute `name` in namespace `ns`.
    pub fn remove(&mut self, ns: &str, name: &str) -> Option<AnyAttribute> {
        let index = self.0.iter().position(|a| a.is(ns, name))?;
        Some(self.0.remove(index))
    }

    /// Removes all attributes in namespace `ns`.
    pub fn remove_namespace(&mut self, ns: &str) {
        self.0.retain(|a| a.ns() != ns);
    }

    /// Iterates over all attributes.
    pub fn iter(&self) -> std::slice::Iter<'_, AnyAttribute> {
        self.0.iter()
    }

    /// Iterates over the attributes in namespace `ns`.
    pub fn in_namespace<'a>(&'a self, ns: &'a str) -> impl Iterator<Item = &'a AnyAttribute> {
        self.0.iter().filter(move |a| a.ns() == ns)
    }

    /// Number of attributes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if there are no attributes.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A node in the content of an [`AnyElement`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnyNode {
    /// A child element.
    Element(AnyElement),
    /// Decoded text content.
    Text(StrResource),
}

/// An XML element with its full subtree, identified by namespace URI and local name.
///
/// Used for `xs:any` slots. The element stored directly in a slot must have a foreign namespace
/// (checked on write); nested elements and attributes may use any namespace, including none.
/// Equality compares namespace, name, attributes and content; the prefix hint is ignored because
/// prefixes carry no meaning in XML.
#[derive(Debug, Clone)]
pub struct AnyElement {
    ns: StrResource,
    name: StrResource,
    prefix_hint: Option<StrResource>,
    attributes: Vec<AnyAttribute>,
    children: Vec<AnyNode>,
}

impl AnyElement {
    /// Creates an empty element. `ns` may be empty for an element without namespace.
    pub fn new(ns: &str, name: &str) -> Result<Self, AnyError> {
        check_name(name)?;
        Ok(Self {
            ns: ns.into(),
            name: name.into(),
            prefix_hint: None,
            attributes: Vec::new(),
            children: Vec::new(),
        })
    }

    /// Sets the preferred prefix used when writing this element.
    pub fn with_prefix_hint(mut self, prefix: &str) -> Self {
        self.prefix_hint = is_ncname(prefix).then(|| prefix.into());
        self
    }

    /// Builder style [`AnyElement::set_attribute`].
    pub fn with_attribute(
        mut self,
        ns: &str,
        name: &str,
        value: impl Into<StrResource>,
    ) -> Result<Self, AnyError> {
        self.set_attribute(ns, name, value)?;
        Ok(self)
    }

    /// Builder style [`AnyElement::push_child`].
    pub fn with_child(mut self, child: AnyElement) -> Self {
        self.push_child(child);
        self
    }

    /// Builder style [`AnyElement::push_text`].
    pub fn with_text(mut self, text: impl Into<StrResource>) -> Self {
        self.push_text(text);
        self
    }

    /// Namespace URI of the element, empty if it has no namespace.
    pub fn ns(&self) -> &str {
        &self.ns
    }

    /// Local name of the element.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns `true` if the element has namespace `ns` and local name `name`.
    pub fn is(&self, ns: &str, name: &str) -> bool {
        &*self.ns == ns && &*self.name == name
    }

    /// Preferred prefix used when writing, if any.
    pub fn prefix_hint(&self) -> Option<&str> {
        self.prefix_hint.as_deref()
    }

    /// Attributes of the element.
    pub fn attributes(&self) -> &[AnyAttribute] {
        &self.attributes
    }

    /// Returns the value of attribute `name` in namespace `ns` (empty for no namespace).
    pub fn attribute(&self, ns: &str, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|a| a.is(ns, name))
            .map(AnyAttribute::value)
    }

    /// Sets attribute `name` in namespace `ns` (empty for no namespace), replacing an existing value.
    pub fn set_attribute(
        &mut self,
        ns: &str,
        name: &str,
        value: impl Into<StrResource>,
    ) -> Result<(), AnyError> {
        let attribute = AnyAttribute::new(ns, name, value)?;
        match self.attributes.iter_mut().find(|a| a.is(ns, name)) {
            Some(existing) => *existing = attribute,
            None => self.attributes.push(attribute),
        }
        Ok(())
    }

    /// Removes and returns attribute `name` in namespace `ns`.
    pub fn remove_attribute(&mut self, ns: &str, name: &str) -> Option<AnyAttribute> {
        let index = self.attributes.iter().position(|a| a.is(ns, name))?;
        Some(self.attributes.remove(index))
    }

    /// Content of the element in document order.
    pub fn children(&self) -> &[AnyNode] {
        &self.children
    }

    /// Mutable access to the content of the element.
    pub fn children_mut(&mut self) -> &mut Vec<AnyNode> {
        &mut self.children
    }

    /// Appends a child element.
    pub fn push_child(&mut self, child: AnyElement) {
        self.children.push(AnyNode::Element(child));
    }

    /// Appends text content.
    pub fn push_text(&mut self, text: impl Into<StrResource>) {
        self.children.push(AnyNode::Text(text.into()));
    }

    /// Iterates over the child elements.
    pub fn child_elements(&self) -> impl Iterator<Item = &AnyElement> {
        self.children.iter().filter_map(|node| match node {
            AnyNode::Element(element) => Some(element),
            AnyNode::Text(_) => None,
        })
    }

    /// Returns the first child element with namespace `ns` and local name `name`.
    pub fn child(&self, ns: &str, name: &str) -> Option<&AnyElement> {
        self.child_elements().find(|e| e.is(ns, name))
    }

    /// Concatenated direct text content of the element.
    pub fn text(&self) -> String {
        self.children
            .iter()
            .filter_map(|node| match node {
                AnyNode::Text(text) => Some(&**text),
                AnyNode::Element(_) => None,
            })
            .collect()
    }
}

impl PartialEq for AnyElement {
    fn eq(&self, other: &Self) -> bool {
        self.ns == other.ns
            && self.name == other.name
            && self.attributes == other.attributes
            && self.children == other.children
    }
}

impl Eq for AnyElement {}

/// Deserializes a foreign element with its subtree. `matches` is always `false`: slots decide
/// themselves which elements are foreign, so derived types never pick an `AnyElement` up.
#[cfg(feature = "read")]
impl<'xml> instant_xml::FromXml<'xml> for AnyElement {
    fn matches(_id: instant_xml::Id<'_>, _field: Option<instant_xml::Id<'_>>) -> bool {
        false
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), instant_xml::Error> {
        if into.is_some() {
            return Err(instant_xml::Error::DuplicateValue(field));
        }
        let id = deserializer.parent();
        *into = Some(read::read_element(deserializer, id)?);
        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: instant_xml::Kind = instant_xml::Kind::Element;
}

/// Serializes a foreign element with its subtree, declaring its namespaces on itself.
#[cfg(feature = "write")]
impl instant_xml::ToXml for AnyElement {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _field: Option<instant_xml::Id<'_>>,
        serializer: &mut instant_xml::Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        write::write_standalone_element(serializer, self)
    }
}

#[cfg(feature = "read")]
pub(crate) mod read {
    use std::borrow::Cow;

    use instant_xml::{Deserializer, Error, Id, de::Node};

    use super::{AnyAttribute, AnyAttributes, AnyElement, AnyNode, is_foreign_for};

    /// Keeps the attribute if it is in a foreign namespace, otherwise ignores it.
    pub(crate) fn capture_attribute(
        attributes: &mut AnyAttributes,
        id: Id<'_>,
        own: &str,
        prefix: Option<&str>,
        value: &str,
    ) {
        if !is_foreign_for(id.ns, own) {
            return;
        }

        // Ignore attributes with names we cannot represent instead of failing the whole read.
        if let Ok(mut attribute) = AnyAttribute::new(id.ns, id.name, value) {
            if let Some(prefix) = prefix {
                attribute = attribute.with_prefix_hint(prefix);
            }
            let _ = attributes.insert(attribute);
        }
    }

    /// Reads the element `id` whose start tag was just consumed, including its whole subtree.
    pub(crate) fn read_element<'xml>(
        deserializer: &mut Deserializer<'_, 'xml>,
        id: Id<'xml>,
    ) -> Result<AnyElement, Error> {
        let mut element =
            AnyElement::new(id.ns, id.name).map_err(|e| Error::Other(e.to_string()))?;

        while let Some(node) = deserializer.next() {
            match node? {
                Node::Attribute(attr) => {
                    let attr_id = deserializer.attribute_id(&attr)?;
                    let mut attribute = AnyAttribute::new(attr_id.ns, attr_id.name, &*attr.value)
                        .map_err(|e| Error::Other(e.to_string()))?;
                    if let Some(prefix) = attr.prefix {
                        attribute = attribute.with_prefix_hint(prefix);
                    }
                    element.attributes.push(attribute);
                }
                Node::Open(child) => {
                    let child_id = deserializer.element_id(&child)?;
                    let mut nested = deserializer.nested(child);
                    let child = read_element(&mut nested, child_id)?;
                    element.children.push(AnyNode::Element(child));
                }
                Node::Text(text) => push_text(&mut element, text),
                Node::AttributeValue(_) | Node::Close { .. } => {}
            }
        }

        Ok(element)
    }

    fn push_text(element: &mut AnyElement, text: Cow<'_, str>) {
        match element.children.last_mut() {
            Some(AnyNode::Text(existing)) => {
                let mut merged = existing.to_string();
                merged.push_str(&text);
                *existing = merged.into();
            }
            _ => element.children.push(AnyNode::Text((&*text).into())),
        }
    }
}

#[cfg(feature = "write")]
pub(crate) mod write {
    use std::fmt;

    use instant_xml::{
        Error, Serializer, ToXml,
        ser::{Context, Element},
    };

    use super::{
        AnyAttribute, AnyAttributes, AnyElement, AnyNode, XML_NS, is_foreign_for,
        is_foreign_namespace, is_ncname,
    };
    use crate::threemf_namespaces::{
        BEAM_LATTICE_BALLS_PREFIX, BEAM_LATTICE_PREFIX, BOOLEAN_PREFIX, CORE_TRIANGLESET_PREFIX,
        DISPLACEMENT_PREFIX, MATERIAL_PREFIX, PROD_PREFIX, SLICE_PREFIX,
    };

    /// Prefixes used by threemf2 for the extensions it models; never assigned to foreign namespaces.
    const RESERVED_PREFIXES: &[&str] = &[
        "xml",
        "xmlns",
        PROD_PREFIX,
        CORE_TRIANGLESET_PREFIX,
        BEAM_LATTICE_PREFIX,
        BEAM_LATTICE_BALLS_PREFIX,
        BOOLEAN_PREFIX,
        SLICE_PREFIX,
        MATERIAL_PREFIX,
        DISPLACEMENT_PREFIX,
    ];

    /// Namespace declarations made for foreign content, valid for the current element subtree.
    #[derive(Clone, Debug)]
    pub(crate) struct ForeignScope {
        default_ns: String,
        prefixes: Vec<(String, String)>,
    }

    impl ForeignScope {
        fn new<W: fmt::Write + ?Sized>(serializer: &Serializer<'_, W>) -> Self {
            Self {
                default_ns: serializer.default_ns().to_owned(),
                prefixes: Vec::new(),
            }
        }

        /// Returns the prefix for `ns` and whether it still has to be declared.
        fn resolve<W: fmt::Write + ?Sized>(
            &mut self,
            serializer: &Serializer<'_, W>,
            ns: &str,
            hint: Option<&str>,
        ) -> (String, bool) {
            if ns == XML_NS {
                return ("xml".to_owned(), false);
            }

            if let Some((prefix, _)) = self.prefixes.iter().rev().find(|(_, uri)| uri == ns) {
                return (prefix.clone(), false);
            }

            if let Some(prefix) = serializer.prefix(ns) {
                return (prefix.to_owned(), false);
            }

            let taken = |p: &str| {
                RESERVED_PREFIXES.contains(&p) || self.prefixes.iter().any(|(q, _)| q == p)
            };

            let prefix = match hint {
                Some(hint) if is_ncname(hint) && !taken(hint) => hint.to_owned(),
                _ => (0..)
                    .map(|i| format!("ns{i}"))
                    .find(|p| !taken(p))
                    .expect("an unused prefix always exists"),
            };

            self.prefixes.push((prefix.clone(), ns.to_owned()));
            (prefix, true)
        }

        /// Assigns prefixes to all namespaces in `names`, giving names with a prefix hint priority,
        /// and returns the `(prefix, namespace)` pairs that have to be declared.
        fn declare_all<'a, W: fmt::Write + ?Sized>(
            &mut self,
            serializer: &Serializer<'_, W>,
            names: impl Iterator<Item = (&'a str, Option<&'a str>)> + Clone,
        ) -> Vec<(String, String)> {
            let mut declarations = Vec::new();
            for hinted in [true, false] {
                for (ns, hint) in names.clone().filter(|(_, hint)| hint.is_some() == hinted) {
                    let (prefix, declare) = self.resolve(serializer, ns, hint);
                    if declare {
                        declarations.push((prefix, ns.to_owned()));
                    }
                }
            }
            declarations
        }
    }

    fn qualified(prefix: &str, name: &str) -> String {
        format!("{prefix}:{name}")
    }

    fn not_foreign(ns: &str) -> Error {
        Error::UnexpectedValue(format!(
            "namespace `{ns}` is not allowed for foreign (##other) content"
        ))
    }

    /// Writes the foreign attributes of the element whose start tag is being written and declares
    /// the namespaces needed by them and by the top-level foreign elements of that element.
    ///
    /// Must be called while the serializer is in the attribute state of that element.
    ///
    /// `declared` lists `(prefix, namespace)` pairs already declared on this start tag, so they
    /// are reused instead of declared twice. `own` is the namespace of that element; foreign
    /// content must use a different one.
    pub(crate) fn write_foreign_attributes<W: fmt::Write + ?Sized>(
        serializer: &mut Serializer<'_, W>,
        declared: &[(String, String)],
        own: &str,
        attributes: &AnyAttributes,
        elements: &[AnyElement],
    ) -> Result<ForeignScope, Error> {
        let mut scope = ForeignScope::new(serializer);
        scope.prefixes.extend(declared.iter().cloned());

        let names = attributes
            .iter()
            .map(|a| (a.ns(), a.prefix_hint()))
            .chain(elements.iter().map(|e| (e.ns(), e.prefix_hint())));
        if let Some((ns, _)) = names.clone().find(|(ns, _)| !is_foreign_for(ns, own)) {
            return Err(not_foreign(ns));
        }

        for (prefix, ns) in scope.declare_all(serializer, names) {
            serializer.write_attr(&qualified("xmlns", &prefix), "", ns.as_str())?;
        }

        for attribute in attributes.iter() {
            write_attribute(serializer, &mut scope, attribute)?;
        }

        Ok(scope)
    }

    fn write_attribute<W: fmt::Write + ?Sized>(
        serializer: &mut Serializer<'_, W>,
        scope: &mut ForeignScope,
        attribute: &AnyAttribute,
    ) -> Result<(), Error> {
        if attribute.ns().is_empty() {
            return serializer.write_attr(attribute.name(), "", attribute.value());
        }

        let (prefix, declare) = scope.resolve(serializer, attribute.ns(), attribute.prefix_hint());
        if declare {
            serializer.write_attr(&qualified("xmlns", &prefix), "", attribute.ns())?;
        }
        serializer.write_attr(&qualified(&prefix, attribute.name()), "", attribute.value())
    }

    /// Declares, on the list element whose start tag is being written, the namespaces used by the
    /// per-entry foreign attributes and top-level child elements in `table`, so entries do not
    /// repeat the declarations.
    ///
    /// Fails if an attribute or element is not foreign to `own` or an index is not smaller than `len`, the
    /// length of the list whose entries are named `entry`.
    pub(crate) fn declare_item_namespaces<W: fmt::Write + ?Sized>(
        serializer: &mut Serializer<'_, W>,
        scope: &mut ForeignScope,
        own: &str,
        table: &impl crate::model::domain::any_extensions::ItemTable,
        len: usize,
        entry: &str,
    ) -> Result<(), Error> {
        if let Some(index) = table.last_index()
            && index >= len
        {
            return Err(Error::UnexpectedValue(format!(
                "foreign content is recorded for `{entry}` {index}, but there are only {len}"
            )));
        }

        let names: Vec<(&str, Option<&str>)> = table
            .entry_parts()
            .flat_map(|(_, attributes, elements)| {
                attributes
                    .iter()
                    .map(|a| (a.ns(), a.prefix_hint()))
                    .chain(elements.iter().map(|e| (e.ns(), e.prefix_hint())))
            })
            .collect();
        if let Some((ns, _)) = names.iter().find(|(ns, _)| !is_foreign_for(ns, own)) {
            return Err(not_foreign(ns));
        }

        for (prefix, ns) in scope.declare_all(serializer, names.iter().copied()) {
            serializer.write_attr(&qualified("xmlns", &prefix), "", ns.as_str())?;
        }
        Ok(())
    }

    /// Writes the foreign attributes of a list entry whose namespaces were declared on the list
    /// element with [`declare_item_namespaces`].
    pub(crate) fn write_item_attributes<W: fmt::Write + ?Sized>(
        serializer: &mut Serializer<'_, W>,
        scope: &ForeignScope,
        attributes: &AnyAttributes,
    ) -> Result<(), Error> {
        for attribute in attributes.iter() {
            let ns = attribute.ns();
            let name = match ns {
                XML_NS => qualified("xml", attribute.name()),
                _ => {
                    let prefix = scope
                        .prefixes
                        .iter()
                        .rev()
                        .find(|(_, uri)| uri == ns)
                        .map(|(prefix, _)| prefix.as_str())
                        .or_else(|| serializer.prefix(ns))
                        .ok_or_else(|| not_foreign(ns))?;
                    qualified(prefix, attribute.name())
                }
            };
            serializer.write_attr(&name, "", attribute.value())?;
        }
        Ok(())
    }

    /// Writes a foreign element outside of any slot (for example as an object shape), declaring
    /// its namespaces on the element itself.
    pub(crate) fn write_standalone_element<W: fmt::Write + ?Sized>(
        serializer: &mut Serializer<'_, W>,
        element: &AnyElement,
    ) -> Result<(), Error> {
        if !is_foreign_namespace(element.ns()) {
            return Err(not_foreign(element.ns()));
        }
        let scope = ForeignScope::new(serializer);
        write_element(serializer, scope, element)
    }

    /// Writes the foreign elements of an `xs:any` slot. Must be called in element content state.
    pub(crate) fn write_foreign_elements<W: fmt::Write + ?Sized>(
        serializer: &mut Serializer<'_, W>,
        scope: &ForeignScope,
        elements: &[AnyElement],
    ) -> Result<(), Error> {
        for element in elements {
            write_element(serializer, scope.clone(), element)?;
        }
        Ok(())
    }

    fn write_element<W: fmt::Write + ?Sized>(
        serializer: &mut Serializer<'_, W>,
        mut scope: ForeignScope,
        element: &AnyElement,
    ) -> Result<(), Error> {
        let element_prefixed = element.ns() != scope.default_ns && !element.ns().is_empty();
        let reset_default = element.ns().is_empty() && !scope.default_ns.is_empty();

        let names = element
            .attributes()
            .iter()
            .filter(|a| !a.ns().is_empty())
            .map(|a| (a.ns(), a.prefix_hint()))
            .chain(element_prefixed.then(|| (element.ns(), element.prefix_hint())));
        let declarations = scope.declare_all(serializer, names);

        let tag = match element_prefixed {
            true => {
                let (prefix, _) = scope.resolve(serializer, element.ns(), None);
                qualified(&prefix, element.name())
            }
            false => element.name().to_owned(),
        };

        // Writing the tag with the serializer's own default namespace makes it emit the name as is.
        let default_ns = serializer.default_ns();
        let start: Element<'_, 0> = serializer.write_start(&tag, default_ns, None::<Context<0>>)?;

        if reset_default {
            serializer.write_attr("xmlns", "", "")?;
            scope.default_ns.clear();
        }

        for (prefix, ns) in &declarations {
            serializer.write_attr(&qualified("xmlns", prefix), "", ns.as_str())?;
        }

        for attribute in element.attributes() {
            write_attribute(serializer, &mut scope, attribute)?;
        }

        if element.children().is_empty() {
            return serializer.end_empty();
        }

        serializer.end_start()?;
        for child in element.children() {
            match child {
                AnyNode::Element(child) => write_element(serializer, scope.clone(), child)?,
                AnyNode::Text(text) => (**text).serialize(None, serializer)?,
            }
        }
        serializer.write_close(start)
    }
}
