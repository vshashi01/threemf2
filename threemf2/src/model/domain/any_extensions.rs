//! Containers for foreign XML content attached to 3MF elements.
//!
//! Every element with an extension point in the 3MF schemas has an always-present field of one of
//! these types:
//!
//! - [`AnyExtensions`] for elements with both `xs:anyAttribute` and an `xs:any` element slot.
//! - [`AnyAttributeExtensions`] for elements that only allow `xs:anyAttribute`.
//! - [`AnyElementExtensions`] for elements that only allow `xs:any` child elements.
//! - [`ItemAttributeExtensions`] and [`ItemAnyExtensions`] on list elements such as `<vertices>`
//!   or `<b:beams>`, for the extension points of their entries, which are too numerous to carry
//!   a field each. The first holds attributes only; the second also holds child elements, for
//!   entries whose schema allows `xs:any` (beams, balls and references).
//!
//! The field exists regardless of features so that struct literals compile the same way in every
//! build. The content is only kept, and the accessors are only available, with the `any` feature.
//! Without it both types are empty and unknown XML content is ignored when reading.

use thiserror::Error;

#[cfg(feature = "any")]
use crate::model::domain::any::{AnyAttributes, AnyElement, is_foreign_namespace};

/// Errors raised when constructing foreign XML content.
///
/// Always available, so that builder error types embedding it have the same shape with and
/// without the `any` feature. Only produced with the `any` feature.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AnyError {
    /// The namespace is not allowed for `##other` content (empty, the 3MF core namespace or `xmlns`).
    #[error("namespace `{0}` is not allowed for foreign (##other) content")]
    NotForeignNamespace(String),

    /// The name is not a valid XML local name (NCName).
    #[error("`{0}` is not a valid XML local name")]
    InvalidName(String),
}

/// Foreign attributes and elements of an element with `xs:anyAttribute` and `xs:any` slots.
///
/// Only populated with the `any` feature; empty and zero-sized otherwise.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnyExtensions {
    #[cfg(feature = "any")]
    pub(crate) attributes: AnyAttributes,
    #[cfg(feature = "any")]
    pub(crate) elements: Vec<AnyElement>,
}

impl AnyExtensions {
    /// Creates an empty container.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "any")]
impl AnyExtensions {
    /// Foreign attributes (`xs:anyAttribute`).
    pub fn attributes(&self) -> &AnyAttributes {
        &self.attributes
    }

    /// Mutable access to the foreign attributes.
    pub fn attributes_mut(&mut self) -> &mut AnyAttributes {
        &mut self.attributes
    }

    /// Foreign child elements (`xs:any`), in document order.
    ///
    /// Each element must have a foreign namespace; this is checked when writing.
    pub fn elements(&self) -> &[AnyElement] {
        &self.elements
    }

    /// Mutable access to the foreign child elements.
    ///
    /// Elements added here are not validated until the model is written; prefer
    /// [`AnyExtensions::push_element`].
    pub fn elements_mut(&mut self) -> &mut Vec<AnyElement> {
        &mut self.elements
    }

    /// Appends a foreign child element.
    ///
    /// # Errors
    ///
    /// Returns [`AnyError::NotForeignNamespace`] if the element is not in a foreign namespace.
    pub fn push_element(&mut self, element: AnyElement) -> Result<(), AnyError> {
        if !is_foreign_namespace(element.ns()) {
            return Err(AnyError::NotForeignNamespace(element.ns().to_owned()));
        }
        self.elements.push(element);
        Ok(())
    }

    /// Builder style: sets the foreign attributes.
    pub fn with_attributes(mut self, attributes: AnyAttributes) -> Self {
        self.attributes = attributes;
        self
    }

    /// Builder style: appends a foreign child element.
    pub fn with_element(mut self, element: AnyElement) -> Self {
        self.elements.push(element);
        self
    }

    /// Returns `true` if there are no foreign attributes and elements.
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty() && self.elements.is_empty()
    }
}

/// Foreign attributes of an element that only has an `xs:anyAttribute` slot.
///
/// Only populated with the `any` feature; empty and zero-sized otherwise.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnyAttributeExtensions {
    #[cfg(feature = "any")]
    pub(crate) attributes: AnyAttributes,
}

impl AnyAttributeExtensions {
    /// Creates an empty container.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "any")]
impl AnyAttributeExtensions {
    /// Foreign attributes (`xs:anyAttribute`).
    pub fn attributes(&self) -> &AnyAttributes {
        &self.attributes
    }

    /// Mutable access to the foreign attributes.
    pub fn attributes_mut(&mut self) -> &mut AnyAttributes {
        &mut self.attributes
    }

    /// Builder style: sets the foreign attributes.
    pub fn with_attributes(mut self, attributes: AnyAttributes) -> Self {
        self.attributes = attributes;
        self
    }

    /// Returns `true` if there are no foreign attributes.
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }
}

/// Common access to [`AnyExtensions`] and [`AnyAttributeExtensions`] for the generated
/// read/write code. Attribute-only slots have no element storage.
#[cfg(feature = "any")]
#[cfg_attr(not(all(feature = "read", feature = "write")), allow(dead_code))]
pub(crate) trait AnySlot {
    fn slot_attributes(&self) -> &AnyAttributes;
    /// `None` if the slot cannot hold attributes.
    fn slot_attributes_mut(&mut self) -> Option<&mut AnyAttributes>;
    fn slot_elements(&self) -> &[AnyElement];
    /// `None` if the slot cannot hold elements.
    fn slot_elements_mut(&mut self) -> Option<&mut Vec<AnyElement>>;
}

#[cfg(feature = "any")]
impl AnySlot for AnyExtensions {
    fn slot_attributes(&self) -> &AnyAttributes {
        &self.attributes
    }
    fn slot_attributes_mut(&mut self) -> Option<&mut AnyAttributes> {
        Some(&mut self.attributes)
    }
    fn slot_elements(&self) -> &[AnyElement] {
        &self.elements
    }
    fn slot_elements_mut(&mut self) -> Option<&mut Vec<AnyElement>> {
        Some(&mut self.elements)
    }
}

#[cfg(feature = "any")]
impl AnySlot for AnyAttributeExtensions {
    fn slot_attributes(&self) -> &AnyAttributes {
        &self.attributes
    }
    fn slot_attributes_mut(&mut self) -> Option<&mut AnyAttributes> {
        Some(&mut self.attributes)
    }
    fn slot_elements(&self) -> &[AnyElement] {
        &[]
    }
    fn slot_elements_mut(&mut self) -> Option<&mut Vec<AnyElement>> {
        None
    }
}

/// Foreign child elements of an element that only has an `xs:any` slot (no `xs:anyAttribute`),
/// such as `<b:beams>`.
///
/// Only populated with the `any` feature; empty and zero-sized otherwise.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnyElementExtensions {
    #[cfg(feature = "any")]
    pub(crate) elements: Vec<AnyElement>,
}

impl AnyElementExtensions {
    /// Creates an empty container.
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(feature = "any")]
impl AnyElementExtensions {
    /// Foreign child elements (`xs:any`), in document order.
    pub fn elements(&self) -> &[AnyElement] {
        &self.elements
    }

    /// Mutable access to the foreign child elements.
    ///
    /// Elements added here are not validated until the model is written; prefer
    /// [`AnyElementExtensions::push_element`].
    pub fn elements_mut(&mut self) -> &mut Vec<AnyElement> {
        &mut self.elements
    }

    /// Appends a foreign child element.
    ///
    /// # Errors
    ///
    /// Returns [`AnyError::NotForeignNamespace`] if the element is not in a foreign namespace.
    pub fn push_element(&mut self, element: AnyElement) -> Result<(), AnyError> {
        if !is_foreign_namespace(element.ns()) {
            return Err(AnyError::NotForeignNamespace(element.ns().to_owned()));
        }
        self.elements.push(element);
        Ok(())
    }

    /// Builder style: appends a foreign child element.
    pub fn with_element(mut self, element: AnyElement) -> Self {
        self.elements.push(element);
        self
    }

    /// Returns `true` if there are no foreign elements.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }
}

#[cfg(feature = "any")]
impl AnySlot for AnyElementExtensions {
    fn slot_attributes(&self) -> &AnyAttributes {
        AnyAttributes::empty()
    }
    fn slot_attributes_mut(&mut self) -> Option<&mut AnyAttributes> {
        None
    }
    fn slot_elements(&self) -> &[AnyElement] {
        &self.elements
    }
    fn slot_elements_mut(&mut self) -> Option<&mut Vec<AnyElement>> {
        Some(&mut self.elements)
    }
}

/// Crate-internal access to the per-entry tables ([`ItemAttributeExtensions`],
/// [`ItemAnyExtensions`]) for the generated read/write code.
#[cfg_attr(
    not(all(feature = "any", feature = "read", feature = "write")),
    allow(dead_code)
)]
pub(crate) trait ItemTable: Default {
    /// Entries as `(index, attributes, elements)`, in index order.
    #[cfg(feature = "any")]
    fn entry_parts(&self) -> impl Iterator<Item = (usize, &AnyAttributes, &[AnyElement])>;

    /// The largest index in the table.
    #[cfg(feature = "any")]
    fn last_index(&self) -> Option<usize>;

    /// Appends content read for the entry at `index`. Reading visits entries in order, so the
    /// table stays sorted. Tables without element storage drop `elements`.
    #[cfg(all(feature = "any", feature = "read"))]
    fn push_read(&mut self, index: usize, attributes: AnyAttributes, elements: Vec<AnyElement>);
}

/// Generates a per-entry table type and its sorted-map API for the entry type `$entry`.
macro_rules! item_table {
    (
        $(#[$meta:meta])*
        $name:ident, $entry:ident, $what:literal
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Default, PartialEq, Eq)]
        pub struct $name {
            #[cfg(feature = "any")]
            pub(crate) entries: Vec<(usize, $entry)>,
        }

        impl $name {
            /// Creates an empty table.
            pub fn new() -> Self {
                Self::default()
            }
        }

        #[cfg(feature = "any")]
        impl $name {
            fn position(&self, index: usize) -> Result<usize, usize> {
                self.entries.binary_search_by_key(&index, |(i, _)| *i)
            }

            #[doc = concat!("Foreign ", $what, " of the entry at `index`, if it has any.")]
            pub fn get(&self, index: usize) -> Option<&$entry> {
                self.position(index).ok().map(|pos| &self.entries[pos].1)
            }

            #[doc = concat!("Mutable access to the foreign ", $what, " of the entry at `index`, if it has any.")]
            pub fn get_mut(&mut self, index: usize) -> Option<&mut $entry> {
                self.position(index)
                    .ok()
                    .map(|pos| &mut self.entries[pos].1)
            }

            #[doc = concat!("Mutable access to the foreign ", $what, " of the entry at `index`, adding an empty")]
            /// value first if it has none.
            pub fn entry(&mut self, index: usize) -> &mut $entry {
                let pos = match self.position(index) {
                    Ok(pos) => pos,
                    Err(pos) => {
                        self.entries.insert(pos, (index, $entry::default()));
                        pos
                    }
                };
                &mut self.entries[pos].1
            }

            #[doc = concat!("Sets the foreign ", $what, " of the entry at `index` and returns the previous value.")]
            pub fn insert(&mut self, index: usize, value: $entry) -> Option<$entry> {
                match self.position(index) {
                    Ok(pos) => Some(std::mem::replace(&mut self.entries[pos].1, value)),
                    Err(pos) => {
                        self.entries.insert(pos, (index, value));
                        None
                    }
                }
            }

            #[doc = concat!("Removes and returns the foreign ", $what, " of the entry at `index`.")]
            pub fn remove(&mut self, index: usize) -> Option<$entry> {
                self.position(index)
                    .ok()
                    .map(|pos| self.entries.remove(pos).1)
            }

            #[doc = concat!("Entries with foreign ", $what, " as `(index, value)`, in index order.")]
            pub fn iter(&self) -> impl Iterator<Item = (usize, &$entry)> {
                self.entries.iter().map(|(index, value)| (*index, value))
            }

            /// Number of entries in the table.
            pub fn len(&self) -> usize {
                self.entries.len()
            }

            /// Returns `true` if the table has no entries.
            pub fn is_empty(&self) -> bool {
                self.entries.is_empty()
            }

            /// Removes all entries.
            pub fn clear(&mut self) {
                self.entries.clear();
            }
        }
    };
}

item_table! {
    /// Foreign attributes of the individual entries of a list element whose entries only allow
    /// `xs:anyAttribute`, such as each `<vertex>` of `<vertices>`.
    ///
    /// These entries are numerous, so their foreign content is not stored on every entry. The
    /// list element keeps one table that maps an entry's index in its list to the attributes of
    /// that entry, and only entries that have foreign attributes appear in it. Indices are kept
    /// sorted.
    ///
    /// The table is not updated when entries are inserted into or removed from the list; adjust
    /// it yourself. Writing fails if an index is not smaller than the length of the list.
    ///
    /// Only populated with the `any` feature; empty and zero-sized otherwise.
    ItemAttributeExtensions, AnyAttributes, "attributes"
}

item_table! {
    /// Foreign attributes and child elements of the individual entries of a list element whose
    /// entries allow both `xs:anyAttribute` and `xs:any`, such as each `<b:beam>` of `<b:beams>`.
    ///
    /// Works like [`ItemAttributeExtensions`], with an [`AnyExtensions`] per entry. Child elements
    /// of an entry must be in a foreign namespace; this is checked when writing.
    ///
    /// Only populated with the `any` feature; empty and zero-sized otherwise.
    ItemAnyExtensions, AnyExtensions, "content"
}

#[cfg(feature = "any")]
impl ItemTable for ItemAttributeExtensions {
    fn entry_parts(&self) -> impl Iterator<Item = (usize, &AnyAttributes, &[AnyElement])> {
        self.entries
            .iter()
            .map(|(index, attributes)| (*index, attributes, &[][..]))
    }

    fn last_index(&self) -> Option<usize> {
        self.entries.last().map(|(index, _)| *index)
    }

    #[cfg(feature = "read")]
    fn push_read(&mut self, index: usize, attributes: AnyAttributes, _elements: Vec<AnyElement>) {
        debug_assert!(self.entries.last().is_none_or(|(last, _)| *last < index));
        if !attributes.is_empty() {
            self.entries.push((index, attributes));
        }
    }
}

#[cfg(feature = "any")]
impl ItemTable for ItemAnyExtensions {
    fn entry_parts(&self) -> impl Iterator<Item = (usize, &AnyAttributes, &[AnyElement])> {
        self.entries
            .iter()
            .map(|(index, content)| (*index, &content.attributes, content.elements.as_slice()))
    }

    fn last_index(&self) -> Option<usize> {
        self.entries.last().map(|(index, _)| *index)
    }

    #[cfg(feature = "read")]
    fn push_read(&mut self, index: usize, attributes: AnyAttributes, elements: Vec<AnyElement>) {
        debug_assert!(self.entries.last().is_none_or(|(last, _)| *last < index));
        self.entries.push((
            index,
            AnyExtensions {
                attributes,
                elements,
            },
        ));
    }
}

#[cfg(not(feature = "any"))]
impl ItemTable for ItemAttributeExtensions {}

#[cfg(not(feature = "any"))]
impl ItemTable for ItemAnyExtensions {}

/// No table, for entries whose schema type allows no foreign content: what is read is dropped
/// and nothing is written.
impl ItemTable for () {
    #[cfg(feature = "any")]
    fn entry_parts(&self) -> impl Iterator<Item = (usize, &AnyAttributes, &[AnyElement])> {
        std::iter::empty()
    }

    #[cfg(feature = "any")]
    fn last_index(&self) -> Option<usize> {
        None
    }

    #[cfg(all(feature = "any", feature = "read"))]
    fn push_read(&mut self, _: usize, _: AnyAttributes, _: Vec<AnyElement>) {}
}

/// Collects the foreign content of one list entry while it is read. Empty and free without the
/// `any` feature.
#[cfg(feature = "read")]
#[derive(Default)]
pub(crate) struct ItemCapture {
    #[cfg(feature = "any")]
    attributes: AnyAttributes,
    #[cfg(feature = "any")]
    elements: Vec<AnyElement>,
}

#[cfg(feature = "read")]
impl ItemCapture {
    /// Keeps the attribute if it is in a namespace foreign to `own`.
    #[inline]
    pub(crate) fn attribute(
        &mut self,
        id: instant_xml::Id<'_>,
        own: &str,
        prefix: Option<&str>,
        value: &str,
    ) {
        #[cfg(feature = "any")]
        crate::model::domain::any::read::capture_attribute(
            &mut self.attributes,
            id,
            own,
            prefix,
            value,
        );
        #[cfg(not(feature = "any"))]
        let _ = (id, own, prefix, value);
    }

    /// Reads the child element `id` whose start tag was just consumed into `nested`, keeping it
    /// if it is in a namespace foreign to `own` and skipping it otherwise.
    #[inline]
    pub(crate) fn element<'xml>(
        &mut self,
        nested: &mut instant_xml::Deserializer<'_, 'xml>,
        id: instant_xml::Id<'xml>,
        own: &str,
    ) -> Result<(), instant_xml::Error> {
        #[cfg(feature = "any")]
        if crate::model::domain::any::is_foreign_for(id.ns, own) {
            let element = crate::model::domain::any::read::read_element(nested, id)?;
            self.elements.push(element);
            return Ok(());
        }
        #[cfg(not(feature = "any"))]
        let _ = (id, own);
        nested.ignore()
    }

    /// Records the captured content, if any, for the entry at `index` of `table`.
    #[inline]
    pub(crate) fn store<T: ItemTable>(self, table: &mut T, index: usize) {
        #[cfg(feature = "any")]
        if !self.attributes.is_empty() || !self.elements.is_empty() {
            table.push_read(index, self.attributes, self.elements);
        }
        #[cfg(not(feature = "any"))]
        let _ = (table, index);
    }
}

/// Generates a `&mut` accessor to a foreign content field of a builder, inside its `impl` block.
/// The accessor only exists with the `any` feature.
///
/// - `content method => field: Type, "tag"`: the foreign content of the element `<tag>`.
/// - `entries method => field: Type, "tag"`: the per-entry table of the `<tag>` entries.
macro_rules! foreign_content_accessor {
    (content $method:ident => $field:ident: $ty:ty, $tag:literal) => {
        #[doc = concat!("Direct access to the foreign content of `<", $tag, ">`.")]
        ///
        /// Content added here is checked when the model is written. Requires the `any` feature.
        #[cfg(feature = "any")]
        pub fn $method(&mut self) -> &mut $ty {
            &mut self.$field
        }
    };
    (entries $method:ident => $field:ident: $ty:ty, $tag:literal) => {
        #[doc = concat!(
            "Foreign content of the individual `<", $tag, ">` entries, by index in the order the ",
            "entries were added."
        )]
        ///
        /// Writing fails if an index is not smaller than the number of entries. Requires the `any`
        /// feature.
        #[cfg(feature = "any")]
        pub fn $method(&mut self) -> &mut $ty {
            &mut self.$field
        }
    };
}

pub(crate) use foreign_content_accessor;

/// Generates getters `pub fn method(&self) -> &'a Type { &self.path }` on a view `View<'a>`;
/// the doc comment above each entry becomes the getter's documentation.
///
/// ```ignore
/// foreign_content_getters!(MeshObjectView {
///     /// Foreign content of the `<mesh>` element.
///     mesh_any_extensions: AnyExtensions = mesh.any_extensions,
/// });
/// ```
#[cfg(feature = "any")]
macro_rules! foreign_content_getters {
    ($view:ident { $($(#[$doc:meta])* $method:ident: $ty:ty = $($path:ident).+,)* }) => {
        impl<'a> $view<'a> {
            $(
                $(#[$doc])*
                pub fn $method(&self) -> &'a $ty {
                    &self.$($path).+
                }
            )*
        }
    };
}

#[cfg(feature = "any")]
pub(crate) use foreign_content_getters;

/// Generates the `any_*` builder methods for a builder with a private
/// `any_extensions: AnyExtensions` field. Pass the generic parameter for generic builders, as in
/// `any_extensions_builder_methods!(ObjectBuilder<T>, T)`.
///
/// The methods return [`AnyError`]; every error type returned from a builder closure converts it
/// with `?`.
#[cfg(feature = "any")]
macro_rules! any_extensions_builder_methods {
    ($builder:ty $(, $generic:ident)?) => {
        impl$(<$generic>)? $builder {
            /// Adds a foreign attribute (`xs:anyAttribute`) `name` in namespace `ns`, replacing
            /// an existing value. Requires the `any` feature.
            ///
            /// # Errors
            ///
            /// Fails if `name` is not a valid XML name or `ns` is not a foreign namespace
            /// (empty or the 3MF core namespace).
            pub fn any_attribute(
                &mut self,
                ns: &str,
                name: &str,
                value: &str,
            ) -> Result<&mut Self, $crate::model::domain::any_extensions::AnyError> {
                self.any_extensions.attributes_mut().set(ns, name, value)?;
                Ok(self)
            }

            /// Appends a foreign child element (`xs:any`). Requires the `any` feature.
            ///
            /// # Errors
            ///
            /// Fails if the element is not in a foreign namespace.
            pub fn any_element(
                &mut self,
                element: $crate::model::domain::any::AnyElement,
            ) -> Result<&mut Self, $crate::model::domain::any_extensions::AnyError> {
                self.any_extensions.push_element(element)?;
                Ok(self)
            }

            /// Direct access to the foreign content of the element being built.
            ///
            /// Content added here is only validated when the model is written.
            /// Requires the `any` feature.
            pub fn any_extensions_mut(
                &mut self,
            ) -> &mut $crate::model::domain::any_extensions::AnyExtensions {
                &mut self.any_extensions
            }
        }
    };
}

#[cfg(feature = "any")]
pub(crate) use any_extensions_builder_methods;

/// Generates the `any_*` query methods for a view type `View<'a>` that has an inherent
/// `fn any_extensions(&self) -> &'a AnyExtensions`.
#[cfg(feature = "any")]
macro_rules! any_extensions_view_methods {
    ($view:ident) => {
        impl<'a> $view<'a> {
            /// Foreign attributes (`xs:anyAttribute`). Requires the `any` feature.
            pub fn any_attributes(&self) -> &'a $crate::model::domain::any::AnyAttributes {
                self.any_extensions().attributes()
            }

            /// Value of the foreign attribute `name` in namespace `ns`, if present.
            /// Requires the `any` feature.
            pub fn any_attribute(&self, ns: &str, name: &str) -> Option<&'a str> {
                self.any_extensions().attributes().get(ns, name)
            }

            /// Foreign child elements (`xs:any`) in document order. Requires the `any` feature.
            pub fn any_elements(&self) -> &'a [$crate::model::domain::any::AnyElement] {
                self.any_extensions().elements()
            }

            /// Foreign child elements in namespace `ns`. Requires the `any` feature.
            pub fn any_elements_in<'n>(
                &self,
                ns: &'n str,
            ) -> impl Iterator<Item = &'a $crate::model::domain::any::AnyElement> + use<'a, 'n>
            {
                self.any_extensions()
                    .elements()
                    .iter()
                    .filter(move |element| element.ns() == ns)
            }
        }
    };
}

#[cfg(feature = "any")]
pub(crate) use any_extensions_view_methods;

#[cfg(test)]
mod tests {
    use super::{
        AnyAttributeExtensions, AnyExtensions, ItemAnyExtensions, ItemAttributeExtensions,
    };

    #[cfg(not(feature = "any"))]
    #[test]
    fn any_extensions_are_zero_sized_without_any_feature() {
        assert_eq!(std::mem::size_of::<AnyExtensions>(), 0);
        assert_eq!(std::mem::size_of::<AnyAttributeExtensions>(), 0);
        assert_eq!(std::mem::size_of::<ItemAttributeExtensions>(), 0);
        assert_eq!(std::mem::size_of::<ItemAnyExtensions>(), 0);
    }

    #[cfg(feature = "any")]
    #[test]
    fn item_attribute_extensions_stay_sorted() {
        use crate::model::domain::any::AnyAttributes;

        let mut table = ItemAttributeExtensions::new();
        table.entry(5).set("urn:a", "x", "5").unwrap();
        table.entry(1).set("urn:a", "x", "1").unwrap();
        table.insert(3, AnyAttributes::new());
        assert_eq!(table.iter().map(|(i, _)| i).collect::<Vec<_>>(), [1, 3, 5]);
        assert_eq!(table.get(5).unwrap().get("urn:a", "x"), Some("5"));
        assert!(table.get(2).is_none());
        assert!(table.remove(3).is_some());
        assert_eq!(table.len(), 2);
    }

    #[cfg(feature = "any")]
    #[test]
    fn any_extensions_start_empty() {
        assert!(AnyExtensions::new().is_empty());
        assert!(AnyAttributeExtensions::new().is_empty());
        assert!(ItemAnyExtensions::new().is_empty());
    }
}
