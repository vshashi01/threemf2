//! Reading and writing of list entries (such as `<vertex>` or `<beam>`) together with the
//! per-entry foreign attributes kept by their list element in an [`ItemAttributeExtensions`].
//!
//! List entries are read and written in hot loops, so they do not carry their own foreign
//! content field. [`XmlItem::read_item`] reports foreign attributes to an [`ItemCapture`] that the
//! list stores under the entry's index, and [`write_items`] writes them back from that table.

#[cfg(feature = "read")]
use instant_xml::Deserializer;
#[cfg(feature = "write")]
use instant_xml::{Id, Serializer, ToXml};

use instant_xml::Error;

use crate::model::domain::any_extensions::ItemAttributeExtensions;
#[cfg(feature = "read")]
use crate::model::domain::any_extensions::ItemCapture;

/// A list entry whose foreign attributes are kept by its list element.
pub(crate) trait XmlItem: Sized {
    /// Local name of the entry element.
    const TAG: &'static str;
    /// Namespace of the entry element.
    const NS: &'static str;

    /// Reads the entry whose start tag was just consumed, passing foreign attributes to
    /// `capture`.
    #[cfg(feature = "read")]
    fn read_item(
        deserializer: &mut Deserializer<'_, '_>,
        capture: &mut ItemCapture,
    ) -> Result<Self, Error>;

    /// Writes the start tag and the schema attributes, exactly as the entry's `ToXml` does,
    /// leaving the start tag open.
    /// Only used for entries with foreign attributes, so unused without the `any` feature.
    #[cfg(feature = "write")]
    #[cfg_attr(not(feature = "any"), allow(dead_code))]
    fn write_open<W: std::fmt::Write + ?Sized>(
        &self,
        serializer: &mut Serializer<'_, W>,
    ) -> Result<(), Error>;
}

/// Implements `FromXml` and [`XmlItem`] for a list entry whose fields are all attributes. Its
/// `ToXml` stays derived; `write_open` writes the same start tag and attributes.
///
/// ```ignore
/// xml_item! {
///     Beam {
///         tag: "beam",
///         ns: BEAM_LATTICE_NS,
///         force_prefix: false,
///         attributes: { v1: ResourceIndex => "v1", r1: Option<f64> => "r1" },
///     }
/// }
/// ```
macro_rules! xml_item {
    (
        $ty:ident {
            tag: $tag:literal,
            ns: $ns:expr,
            force_prefix: $force:literal,
            attributes: { $($afield:ident: $aty:ty => $aname:literal),* $(,)? },
        }
    ) => {
        #[cfg(feature = "read")]
        impl<'xml> ::instant_xml::FromXml<'xml> for $ty {
            #[inline]
            fn matches(id: ::instant_xml::Id<'_>, _field: Option<::instant_xml::Id<'_>>) -> bool {
                id == ::instant_xml::Id { ns: $ns, name: $tag }
            }

            fn deserialize<'cx>(
                into: &mut Self::Accumulator,
                field: &'static str,
                deserializer: &mut ::instant_xml::Deserializer<'cx, 'xml>,
            ) -> ::std::result::Result<(), ::instant_xml::Error> {
                if into.is_some() {
                    return Err(::instant_xml::Error::DuplicateValue(field));
                }
                *into = Some(<Self as $crate::model::domain::xml_item::XmlItem>::read_item(
                    deserializer,
                    &mut $crate::model::domain::any_extensions::ItemCapture::default(),
                )?);
                Ok(())
            }

            type Accumulator = Option<Self>;
            const KIND: ::instant_xml::Kind = ::instant_xml::Kind::Element;
        }

        impl $crate::model::domain::xml_item::XmlItem for $ty {
            const TAG: &'static str = $tag;
            const NS: &'static str = $ns;

            #[cfg(feature = "read")]
            fn read_item(
                deserializer: &mut ::instant_xml::Deserializer<'_, '_>,
                capture: &mut $crate::model::domain::any_extensions::ItemCapture,
            ) -> ::std::result::Result<Self, ::instant_xml::Error> {
                #[allow(unused_imports)]
                use ::instant_xml::{Accumulate, FromXml, de::Node};

                $(let mut $afield = <$aty as FromXml<'_>>::Accumulator::default();)*

                while let Some(node) = deserializer.next() {
                    match node? {
                        Node::Attribute(attr) => {
                            let id = deserializer.attribute_id(&attr)?;
                            if !id.ns.is_empty() {
                                capture.attribute(id, $ns, attr.prefix, &attr.value);
                                continue;
                            }
                            $(
                                if id.name == $aname {
                                    $crate::model::domain::xml_helpers::read_attribute::<$aty>(
                                        &mut $afield,
                                        stringify!($afield),
                                        deserializer,
                                        attr.value,
                                    )?;
                                    continue;
                                }
                            )*
                        }
                        Node::Open(element) => deserializer.nested(element).ignore()?,
                        _ => {}
                    }
                }

                Ok(Self {
                    $($afield: $afield.try_done(stringify!($afield))?,)*
                })
            }

            #[cfg(feature = "write")]
            fn write_open<W: ::std::fmt::Write + ?Sized>(
                &self,
                serializer: &mut ::instant_xml::Serializer<'_, W>,
            ) -> ::std::result::Result<(), ::instant_xml::Error> {
                #[allow(unused_imports)]
                use ::instant_xml::ToXml;

                let mut context = ::instant_xml::ser::Context::<0>::default();
                context.default_ns = $ns;
                context.force_prefix = $force;
                serializer.write_start($tag, $ns, Some(context))?;
                $(
                    if ToXml::present(&self.$afield) {
                        serializer.write_attr($aname, "", &self.$afield)?;
                    }
                )*
                Ok(())
            }
        }
    };
}

pub(crate) use xml_item;

/// Reads one entry and appends it to `items`, recording its foreign attributes in `table`.
#[cfg(feature = "read")]
#[inline]
pub(crate) fn read_item_into<T: XmlItem>(
    deserializer: &mut Deserializer<'_, '_>,
    items: &mut Vec<T>,
    table: &mut ItemAttributeExtensions,
) -> Result<(), Error> {
    let mut capture = ItemCapture::default();
    let item = T::read_item(deserializer, &mut capture)?;
    capture.store(table, items.len());
    items.push(item);
    Ok(())
}

/// Writes the entries of a list and the foreign attributes recorded for them in `table`.
///
/// With the `any` feature, the namespaces must have been declared on the list element with
/// [`declare_item_namespaces`](crate::model::domain::any::write::declare_item_namespaces), which
/// returned `scope`.
#[cfg(feature = "write")]
pub(crate) fn write_items<T: XmlItem + ToXml, W: std::fmt::Write + ?Sized>(
    serializer: &mut Serializer<'_, W>,
    items: &[T],
    table: &ItemAttributeExtensions,
    #[cfg(feature = "any")] scope: &crate::model::domain::any::write::ForeignScope,
) -> Result<(), Error> {
    let id = Some(Id {
        ns: T::NS,
        name: T::TAG,
    });

    #[cfg(feature = "any")]
    if !table.entries.is_empty() {
        let mut entries = table.entries.iter().peekable();
        for (index, item) in items.iter().enumerate() {
            match entries.next_if(|(i, _)| *i == index) {
                Some((_, attributes)) if !attributes.is_empty() => {
                    item.write_open(serializer)?;
                    crate::model::domain::any::write::write_item_attributes(
                        serializer, scope, attributes,
                    )?;
                    serializer.end_empty()?;
                }
                _ => item.serialize(id, serializer)?,
            }
        }
        return Ok(());
    }

    #[cfg(not(feature = "any"))]
    let _ = table;

    for item in items {
        item.serialize(id, serializer)?;
    }
    Ok(())
}
