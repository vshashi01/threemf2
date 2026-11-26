//! Containers for foreign XML content attached to 3MF elements.
//!
//! Every element with an extension point in the 3MF schemas has an always-present field of one of
//! these types:
//!
//! - [`AnyExtensions`] for elements with both `xs:anyAttribute` and an `xs:any` element slot.
//! - [`AnyAttributeExtensions`] for elements that only allow `xs:anyAttribute`.
//!
//! The field exists regardless of features so that struct literals compile the same way in every
//! build. The content is only kept, and the accessors are only available, with the `any` feature.
//! Without it both types are empty and unknown XML content is ignored when reading.

#[cfg(feature = "any")]
use crate::model::domain::any::{AnyAttributes, AnyElement, AnyError, is_foreign_namespace};

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

/// Generates the `any_*` builder methods for a builder with the private fields
/// `any_extensions: AnyExtensions` and `any_extensions_error: Option<AnyError>`.
///
/// Builder closures return `()`, so the methods never fail directly: the first error is kept in
/// `any_extensions_error` and the builder's `build` reports it.
#[cfg(feature = "any")]
macro_rules! any_extensions_builder_methods {
    ($builder:ty) => {
        impl $builder {
            /// Adds a foreign attribute (`xs:anyAttribute`) `name` in namespace `ns`, replacing
            /// an existing value.
            ///
            /// An invalid name, or a namespace that is not foreign (empty or the 3MF core
            /// namespace), makes building fail. Requires the `any` feature.
            pub fn any_attribute(&mut self, ns: &str, name: &str, value: &str) -> &mut Self {
                if let Err(err) = self.any_extensions.attributes_mut().set(ns, name, value) {
                    self.any_extensions_error.get_or_insert(err);
                }
                self
            }

            /// Appends a foreign child element (`xs:any`).
            ///
            /// An element that is not in a foreign namespace makes building fail.
            /// Requires the `any` feature.
            pub fn any_element(
                &mut self,
                element: $crate::model::domain::any::AnyElement,
            ) -> &mut Self {
                if let Err(err) = self.any_extensions.push_element(element) {
                    self.any_extensions_error.get_or_insert(err);
                }
                self
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
    use super::{AnyAttributeExtensions, AnyExtensions};

    #[cfg(not(feature = "any"))]
    #[test]
    fn any_extensions_are_zero_sized_without_any_feature() {
        assert_eq!(std::mem::size_of::<AnyExtensions>(), 0);
        assert_eq!(std::mem::size_of::<AnyAttributeExtensions>(), 0);
    }

    #[cfg(feature = "any")]
    #[test]
    fn any_extensions_start_empty() {
        assert!(AnyExtensions::new().is_empty());
        assert!(AnyAttributeExtensions::new().is_empty());
    }
}
