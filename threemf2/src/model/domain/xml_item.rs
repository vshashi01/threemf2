//! Reading and writing of list entries (such as `<vertex>` or `<beam>`) together with the
//! per-entry foreign content kept by their list element in an
//! [`ItemAttributeExtensions`](crate::model::domain::any_extensions::ItemAttributeExtensions) or
//! [`ItemAnyExtensions`](crate::model::domain::any_extensions::ItemAnyExtensions).
//!
//! List entries are read and written in hot loops, so they do not carry their own foreign
//! content field. [`XmlItem::read_item`] reports foreign content to an [`ItemCapture`] that the
//! list stores under the entry's index, and [`write_items`] writes it back from that table.

#[cfg(feature = "read")]
use instant_xml::Deserializer;
#[cfg(feature = "write")]
use instant_xml::{Id, Serializer, ToXml};

use instant_xml::Error;

#[cfg(feature = "read")]
use crate::model::domain::any_extensions::ItemCapture;
use crate::model::domain::any_extensions::ItemTable;

/// A list entry whose foreign content is kept by its list element.
pub(crate) trait XmlItem: Sized {
    /// Local name of the entry element.
    const TAG: &'static str;
    /// Namespace of the entry element.
    const NS: &'static str;

    /// Reads the entry whose start tag was just consumed, passing foreign content to `capture`.
    #[cfg(feature = "read")]
    fn read_item(
        deserializer: &mut Deserializer<'_, '_>,
        capture: &mut ItemCapture,
    ) -> Result<Self, Error>;

    /// Writes the start tag and the schema attributes, exactly as the entry's `ToXml` does,
    /// leaving the start tag open, and returns the element to close.
    ///
    /// Only used for entries with foreign content, so unused without the `any` feature.
    #[cfg(feature = "write")]
    #[cfg_attr(not(feature = "any"), allow(dead_code))]
    fn write_open<W: std::fmt::Write + ?Sized>(
        &self,
        serializer: &mut Serializer<'_, W>,
    ) -> Result<instant_xml::ser::Element<'static, 0>, Error>;
}

/// Implements `FromXml` and [`XmlItem`] for a list entry whose fields are all attributes. Its
/// `ToXml` stays derived; `write_open` writes the same start tag and attributes. Foreign child
/// elements are passed to the capture, which keeps them for tables with element storage.
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
                        Node::Open(element) => {
                            let id = deserializer.element_id(&element)?;
                            let mut nested = deserializer.nested(element);
                            capture.element(&mut nested, id, $ns)?;
                        }
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
            ) -> ::std::result::Result<::instant_xml::ser::Element<'static, 0>, ::instant_xml::Error>
            {
                #[allow(unused_imports)]
                use ::instant_xml::ToXml;

                let mut context = ::instant_xml::ser::Context::<0>::default();
                context.default_ns = $ns;
                context.force_prefix = $force;
                let element = serializer.write_start($tag, $ns, Some(context))?;
                $(
                    if ToXml::present(&self.$afield) {
                        serializer.write_attr($aname, "", &self.$afield)?;
                    }
                )*
                Ok(element)
            }
        }
    };
}

pub(crate) use xml_item;

/// Implements `FromXml` and [`XmlItem`] for a numerous list entry (such as `<vertex>`) with a
/// lenient fast reader: values are parsed with `lexical_core`, and a missing or unparsable value
/// falls back to its default instead of failing the entry, so list indices never shift. Its
/// `ToXml` stays derived; `write_open` writes the same start tag and attributes.
///
/// Each attribute is `field: Type = kind`, where `kind` is one of:
/// - `number`: a [`Double`](crate::model::Double), `0` if missing or invalid.
/// - `index`: a plain integer, `0` if missing or invalid.
/// - `optional`: an `OptionalResourceIndex` / `OptionalResourceId`, `none()` if missing or
///   invalid.
/// - `bytes`: a type built with `From<&[u8]>` from the raw value, `none()` if missing.
///
/// ```ignore
/// fast_item! {
///     Vertex {
///         tag: "vertex",
///         ns: CORE_NS,
///         force_prefix: false,
///         attributes: { x: Double = number, y: Double = number, z: Double = number },
///     }
/// }
/// ```
macro_rules! fast_item {
    (@init number $ty:ty) => { 0.0_f64 };
    (@init index $ty:ty) => { <$ty as ::std::default::Default>::default() };
    (@init optional $ty:ty) => { <$ty>::none() };
    (@init bytes $ty:ty) => { <$ty>::none() };

    (@assign number $ty:ty, $var:ident, $value:expr) => {
        $var = ::lexical_core::parse($value).unwrap_or_default()
    };
    (@assign index $ty:ty, $var:ident, $value:expr) => {
        $var = ::lexical_core::parse($value).unwrap_or_default()
    };
    (@assign optional $ty:ty, $var:ident, $value:expr) => {
        if let Ok(value) = ::lexical_core::parse($value) {
            $var = <$ty>::new(value);
        }
    };
    (@assign bytes $ty:ty, $var:ident, $value:expr) => {
        $var = $value.into()
    };
    (@init checked $ty:ty) => { <$ty as ::std::default::Default>::default() };
    (@assign checked $ty:ty, $var:ident, $value:expr) => {
        $var = ::lexical_core::parse($value).map_err(|_| {
            ::instant_xml::Error::MissingValue(concat!("Invalid values as ", stringify!($var)))
        })?
    };

    (@done number $var:ident) => { $crate::model::Double::new($var) };
    (@done $kind:ident $var:ident) => { $var };

    (
        $ty:ident {
            tag: $tag:literal,
            ns: $ns:expr,
            force_prefix: $force:literal,
            attributes: { $($afield:ident: $aty:ty = $kind:ident),* $(,)? },
        }
    ) => {
        #[cfg(feature = "read")]
        impl<'xml> ::instant_xml::FromXml<'xml> for $ty {
            #[inline]
            fn matches(id: ::instant_xml::Id<'_>, _: Option<::instant_xml::Id<'_>>) -> bool {
                id == ::instant_xml::Id { ns: $ns, name: $tag }
            }

            fn deserialize<'cx>(
                into: &mut Self::Accumulator,
                _: &'static str,
                deserializer: &mut ::instant_xml::Deserializer<'cx, 'xml>,
            ) -> ::std::result::Result<(), ::instant_xml::Error> {
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
                use ::instant_xml::de::Node;

                $(let mut $afield = $crate::model::domain::xml_item::fast_item!(@init $kind $aty);)*

                while let Some(node) = deserializer.next() {
                    match node? {
                        Node::Attribute(attr) => {
                            let id = deserializer.attribute_id(&attr)?;
                            // Schema attributes are unqualified; prefixed ones belong to other
                            // namespaces.
                            if !id.ns.is_empty() {
                                capture.attribute(id, $ns, attr.prefix, &attr.value);
                                continue;
                            }

                            match id.name {
                                $(
                                    stringify!($afield) => {
                                        $crate::model::domain::xml_item::fast_item!(
                                            @assign $kind $aty, $afield, attr.value.as_bytes()
                                        );
                                    }
                                )*
                                _ => {}
                            }
                        }
                        Node::Open(data) => {
                            let mut nested = deserializer.nested(data);
                            nested.ignore()?;
                        }
                        Node::Text(_) => {}
                        _ => {
                            return Err(::instant_xml::Error::UnexpectedNode(
                                "Unexpected".to_owned(),
                            ));
                        }
                    }
                }

                Ok(Self {
                    $($afield: $crate::model::domain::xml_item::fast_item!(@done $kind $afield),)*
                })
            }

            #[cfg(feature = "write")]
            fn write_open<W: ::std::fmt::Write + ?Sized>(
                &self,
                serializer: &mut ::instant_xml::Serializer<'_, W>,
            ) -> ::std::result::Result<::instant_xml::ser::Element<'static, 0>, ::instant_xml::Error>
            {
                #[allow(unused_imports)]
                use ::instant_xml::ToXml;

                let mut context = ::instant_xml::ser::Context::<0>::default();
                context.default_ns = $ns;
                context.force_prefix = $force;
                let element = serializer.write_start($tag, $ns, Some(context))?;
                $(
                    if ToXml::present(&self.$afield) {
                        serializer.write_attr(stringify!($afield), "", &self.$afield)?;
                    }
                )*
                Ok(element)
            }
        }
    };
}

pub(crate) use fast_item;

/// Implements `FromXml` and `ToXml` for the list element of a numerous entry type (such as
/// `<vertices>`), read leniently: malformed entries are skipped and any other child element is
/// read as an entry. The entries are collected into a buffer preallocated with `capacity`.
///
/// - `attributes`: the list element's own attributes, as `field: Type = kind` with the kinds of
///   [`fast_item!`], plus `checked` (a `lexical_core` integer that fails the list if invalid).
///   Empty values are ignored.
/// - `items`: the entry field, its [`XmlItem`] type and the table of their foreign content, if
///   its schema type allows any (`field: Type => table`; without `=> table`, foreign content of the
///   entries is dropped).
/// - `any` (optional): the list element's own foreign content. With an `AnyExtensions`, foreign
///   child elements are kept there instead of being read as entries.
///
/// ```ignore
/// xml_list! {
///     Triangles {
///         tag: "triangles",
///         ns: DISPLACEMENT_NS,
///         force_prefix: false,
///         attributes: { did: OptionalResourceId = bytes },
///         items: triangle: Triangle => triangle_any_extensions,
///         capacity: constants::MAX_TRIANGLE_BUFFER,
///         any: any_extensions: AnyAttributeExtensions,
///     }
/// }
/// ```
macro_rules! xml_list {
    (
        $ty:ident {
            tag: $tag:literal,
            ns: $ns:expr,
            force_prefix: $force:literal,
            attributes: { $($afield:ident: $aty:ty = $kind:ident),* $(,)? },
            items: $ifield:ident: $ity:ty $(=> $itable:ident)?,
            capacity: $capacity:expr,
            $(any: $any:ident: $anyty:ty,)?
        }
    ) => {
        #[cfg(feature = "read")]
        impl<'xml> ::instant_xml::FromXml<'xml> for $ty {
            #[inline]
            fn matches(id: ::instant_xml::Id<'_>, _: Option<::instant_xml::Id<'_>>) -> bool {
                id == ::instant_xml::Id { ns: $ns, name: $tag }
            }

            fn deserialize<'cx>(
                into: &mut Self::Accumulator,
                field: &'static str,
                deserializer: &mut ::instant_xml::Deserializer<'cx, 'xml>,
            ) -> ::std::result::Result<(), ::instant_xml::Error> {
                use ::instant_xml::de::Node;

                if into.is_some() {
                    return Err(::instant_xml::Error::DuplicateValue(field));
                }

                $(let mut $afield = $crate::model::domain::xml_item::fast_item!(@init $kind $aty);)*
                let mut $ifield: ::std::vec::Vec<$ity> = ::std::vec::Vec::with_capacity($capacity);
                let mut no_table = ();
                #[allow(unused_variables)]
                let table = &mut no_table;
                $(
                    let mut $itable = ::std::default::Default::default();
                    let table = &mut $itable;
                )?
                $(
                    #[allow(unused_mut)]
                    let mut $any = <$anyty>::default();
                )?

                while let Some(node) = deserializer.next() {
                    let Ok(node) = node else { continue };
                    match node {
                        #[allow(unused_variables)]
                        Node::Attribute(attr) => {
                            let Ok(id) = deserializer.attribute_id(&attr) else { continue };
                            if id.ns.is_empty() && !attr.value.is_empty() {
                                #[allow(clippy::match_single_binding)]
                                match id.name {
                                    $(
                                        stringify!($afield) => {
                                            $crate::model::domain::xml_item::fast_item!(
                                                @assign $kind $aty, $afield, attr.value.as_bytes()
                                            );
                                            continue;
                                        }
                                    )*
                                    _ => {}
                                }
                            }
                            $(
                                #[cfg(feature = "any")]
                                if let Some(attributes) =
                                    $crate::model::domain::any_extensions::AnySlot::slot_attributes_mut(
                                        &mut $any,
                                    )
                                {
                                    $crate::model::domain::any::read::capture_attribute(
                                        attributes,
                                        id,
                                        $ns,
                                        attr.prefix,
                                        &attr.value,
                                    );
                                }
                            )?
                        }
                        Node::Open(element) => {
                            $(
                                #[cfg(feature = "any")]
                                if let Some(elements) =
                                    $crate::model::domain::any_extensions::AnySlot::slot_elements_mut(
                                        &mut $any,
                                    )
                                {
                                    let id = deserializer.element_id(&element)?;
                                    if $crate::model::domain::any::is_foreign_for(id.ns, $ns) {
                                        let mut nested = deserializer.nested(element);
                                        elements.push(
                                            $crate::model::domain::any::read::read_element(
                                                &mut nested,
                                                id,
                                            )?,
                                        );
                                        continue;
                                    }
                                }
                            )?
                            let mut nested = deserializer.nested(element);
                            // Malformed entries are skipped.
                            let _ = $crate::model::domain::xml_item::read_item_into(
                                &mut nested,
                                &mut $ifield,
                                &mut *table,
                            );
                        }
                        _ => {}
                    }
                }

                $ifield.shrink_to_fit();
                *into = Some(Self {
                    $($afield: $crate::model::domain::xml_item::fast_item!(@done $kind $afield),)*
                    $ifield,
                    $($itable,)?
                    $($any,)?
                });
                Ok(())
            }

            type Accumulator = Option<Self>;
            const KIND: ::instant_xml::Kind = ::instant_xml::Kind::Element;
        }

        #[cfg(feature = "write")]
        impl ::instant_xml::ToXml for $ty {
            fn serialize<W: ::std::fmt::Write + ?Sized>(
                &self,
                _: Option<::instant_xml::Id<'_>>,
                serializer: &mut ::instant_xml::Serializer<W>,
            ) -> ::std::result::Result<(), ::instant_xml::Error> {
                #[allow(unused_imports)]
                use ::instant_xml::ToXml;

                let mut context = ::instant_xml::ser::Context::<0>::default();
                context.default_ns = $ns;
                context.force_prefix = $force;
                let element = serializer.write_start($tag, $ns, Some(context))?;
                $(
                    if ToXml::present(&self.$afield) {
                        serializer.write_attr(stringify!($afield), "", &self.$afield)?;
                    }
                )*

                #[allow(unused_variables)]
                let table = &();
                $(let table = &self.$itable;)?

                #[cfg(feature = "any")]
                #[allow(unused_variables)]
                let (foreign_attributes, foreign_elements) = (
                    $crate::model::domain::any::AnyAttributes::empty(),
                    &[] as &[$crate::model::domain::any::AnyElement],
                );
                $(
                    #[cfg(feature = "any")]
                    let (foreign_attributes, foreign_elements) = (
                        $crate::model::domain::any_extensions::AnySlot::slot_attributes(&self.$any),
                        $crate::model::domain::any_extensions::AnySlot::slot_elements(&self.$any),
                    );
                )?
                #[cfg(feature = "any")]
                let mut scope = $crate::model::domain::any::write::write_foreign_attributes(
                    serializer,
                    &[],
                    $ns,
                    foreign_attributes,
                    foreign_elements,
                )?;
                #[cfg(feature = "any")]
                $crate::model::domain::any::write::declare_item_namespaces(
                    serializer,
                    &mut scope,
                    $ns,
                    table,
                    self.$ifield.len(),
                    <$ity as $crate::model::domain::xml_item::XmlItem>::TAG,
                )?;
                serializer.end_start()?;

                #[cfg(feature = "any")]
                $crate::model::domain::xml_item::write_items(
                    serializer,
                    &self.$ifield,
                    table,
                    &scope,
                )?;
                #[cfg(not(feature = "any"))]
                $crate::model::domain::xml_item::write_items(
                    serializer,
                    &self.$ifield,
                    table,
                )?;
                #[cfg(feature = "any")]
                $crate::model::domain::any::write::write_foreign_elements(
                    serializer,
                    &scope,
                    foreign_elements,
                )?;
                serializer.write_close(element)
            }
        }
    };
}

pub(crate) use xml_list;

/// Reads one entry and appends it to `items`, recording its foreign content in `table`.
#[cfg(feature = "read")]
#[inline]
pub(crate) fn read_item_into<T: XmlItem, Tb: ItemTable>(
    deserializer: &mut Deserializer<'_, '_>,
    items: &mut Vec<T>,
    table: &mut Tb,
) -> Result<(), Error> {
    let mut capture = ItemCapture::default();
    let item = T::read_item(deserializer, &mut capture)?;
    capture.store(table, items.len());
    items.push(item);
    Ok(())
}

/// Writes the entries of a list and the foreign content recorded for them in `table`.
///
/// With the `any` feature, the namespaces must have been declared on the list element with
/// [`declare_item_namespaces`](crate::model::domain::any::write::declare_item_namespaces), which
/// returned `scope`.
#[cfg(feature = "write")]
pub(crate) fn write_items<T: XmlItem + ToXml, Tb: ItemTable, W: std::fmt::Write + ?Sized>(
    serializer: &mut Serializer<'_, W>,
    items: &[T],
    table: &Tb,
    #[cfg(feature = "any")] scope: &crate::model::domain::any::write::ForeignScope,
) -> Result<(), Error> {
    let id = Some(Id {
        ns: T::NS,
        name: T::TAG,
    });

    #[cfg(feature = "any")]
    if table.last_index().is_some() {
        use crate::model::domain::any::write::{write_foreign_elements, write_item_attributes};

        let mut entries = table.entry_parts().peekable();
        for (index, item) in items.iter().enumerate() {
            match entries.next_if(|(i, _, _)| *i == index) {
                Some((_, attributes, elements))
                    if !attributes.is_empty() || !elements.is_empty() =>
                {
                    let element = item.write_open(serializer)?;
                    write_item_attributes(serializer, scope, attributes)?;
                    if elements.is_empty() {
                        serializer.end_empty()?;
                    } else {
                        serializer.end_start()?;
                        write_foreign_elements(serializer, scope, elements)?;
                        serializer.write_close(element)?;
                    }
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
