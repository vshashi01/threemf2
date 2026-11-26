//! `xml_struct!` generates `FromXml` and `ToXml` for 3MF element types that carry foreign
//! content (`xs:anyAttribute` / `xs:any`).
//!
//! The instant-xml derive drops unknown attributes and elements and has no hook to keep them, so
//! these types are implemented by this macro instead. The generated code produces the same XML
//! as the derive for the listed fields, and with the `any` feature it also reads and writes the
//! foreign content stored in the type's `AnyExtensions` / `AnyAttributeExtensions` field.
//!
//! ```ignore
//! xml_struct! {
//!     Build {
//!         tag: "build",
//!         ns: CORE_NS,
//!         prefixes: [(PROD_PREFIX, PROD_NS)],
//!         force_prefix: false,
//!         attributes: { uuid: Option<UuidResource> => (PROD_NS, "UUID") },
//!         elements: { item: Vec<Item> => (CORE_NS, "item") },
//!         any: any_extensions: AnyAttributeExtensions,
//!     }
//! }
//! ```
//!
//! - `attributes`: `field: Type => (namespace, name)`; use `""` for attributes without namespace.
//! - `elements`: child element fields, written in the listed order.
//! - `items` (optional): list entry fields as `field: Type => table`, where `Type` implements
//!   [`XmlItem`](crate::model::domain::xml_item::XmlItem) and `table` is the
//!   `ItemAttributeExtensions` field holding the foreign attributes of the entries. Written after
//!   `elements`.
//! - `any`: the field holding the foreign content.
//! - `shape: field => Variant` (optional): an `Option<Enum>` element field whose foreign
//!   alternative (`Variant(AnyElement)`) takes the first foreign element when no other value was
//!   read, as for the shape of a 3MF object.
//! - `start_hook: path` (optional): called right after the start tag is opened with
//!   `(&self, &mut Serializer)`; returns the `(prefix, namespace)` pairs it declared.
//! - `self_closing: true` (optional): write `<tag ... />` when no child element is present
//!   (`None` optional children are skipped). The default matches the derive, which writes an
//!   open/close pair whenever the type has child element fields.

macro_rules! xml_struct {
    (@self_closing) => {
        false
    };
    (@self_closing $value:literal) => {
        $value
    };
    (
        $ty:ident {
            tag: $tag:literal,
            ns: $ns:expr,
            prefixes: [$(($pfx:expr, $pns:expr)),* $(,)?],
            force_prefix: $force:literal,
            attributes: { $($afield:ident: $aty:ty => ($ans:expr, $aname:literal)),* $(,)? },
            elements: { $($efield:ident: $ety:ty => ($ens:expr, $ename:literal)),* $(,)? },
            $(items: { $($ifield:ident: $ity:ty => $itable:ident),* $(,)? },)?
            any: $any:ident: $anyty:ty,
            $(shape: $shape:ident => $variant:path,)?
            $(start_hook: $hook:path,)?
            $(self_closing: $self_closing:literal,)?
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
                #[allow(unused_imports)]
                use ::instant_xml::{Accumulate, FromXml, de::Node};

                if into.is_some() {
                    return Err(::instant_xml::Error::DuplicateValue(field));
                }

                $(let mut $afield = <$aty as FromXml<'xml>>::Accumulator::default();)*
                $(let mut $efield = <$ety as FromXml<'xml>>::Accumulator::default();)*
                $($(
                    let mut $ifield: ::std::vec::Vec<$ity> = ::std::vec::Vec::new();
                    let mut $itable =
                        $crate::model::domain::any_extensions::ItemAttributeExtensions::default();
                )*)?
                #[allow(unused_mut)]
                let mut $any = <$anyty>::default();

                while let Some(node) = deserializer.next() {
                    match node? {
                        Node::Attribute(attr) => {
                            #[allow(unused_variables)]
                            let id = deserializer.attribute_id(&attr)?;
                            $(
                                // An unprefixed attribute also matches a field declared in the
                                // element's own namespace, as the derive allows; it is written
                                // unprefixed too.
                                if id.name == $aname
                                    && (id.ns == $ans || (id.ns.is_empty() && $ans == $ns))
                                {
                                    $crate::model::domain::xml_helpers::read_attribute::<$aty>(
                                        &mut $afield,
                                        stringify!($afield),
                                        deserializer,
                                        attr.value,
                                    )?;
                                    continue;
                                }
                            )*
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
                        }
                        Node::Open(element) => {
                            #[allow(unused_variables)]
                            let id = deserializer.element_id(&element)?;
                            $($(
                                if id.ns == <$ity as $crate::model::domain::xml_item::XmlItem>::NS
                                    && id.name == <$ity as $crate::model::domain::xml_item::XmlItem>::TAG
                                {
                                    let mut nested = deserializer.nested(element);
                                    $crate::model::domain::xml_item::read_item_into(
                                        &mut nested,
                                        &mut $ifield,
                                        &mut $itable,
                                    )?;
                                    continue;
                                }
                            )*)?
                            $(
                                if <$ety as FromXml<'xml>>::matches(
                                    id,
                                    Some(::instant_xml::Id { ns: $ens, name: $ename }),
                                ) {
                                    let mut nested = deserializer.nested(element);
                                    <$ety as FromXml<'xml>>::deserialize(
                                        &mut $efield,
                                        stringify!($efield),
                                        &mut nested,
                                    )?;
                                    continue;
                                }
                            )*
                            let mut nested = deserializer.nested(element);

                            #[cfg(feature = "any")]
                            if $crate::model::domain::any::is_foreign_for(id.ns, $ns) {
                                let element =
                                    $crate::model::domain::any::read::read_element(&mut nested, id)?;
                                $(
                                    if $shape.get_mut().is_none() {
                                        *$shape.get_mut() = Some($variant(element));
                                        continue;
                                    }
                                )?
                                if let Some(elements) =
                                    $crate::model::domain::any_extensions::AnySlot::slot_elements_mut(
                                        &mut $any,
                                    )
                                {
                                    elements.push(element);
                                }
                                continue;
                            }

                            nested.ignore()?;
                        }
                        _ => {}
                    }
                }

                *into = Some(Self {
                    $($afield: $afield.try_done(stringify!($afield))?,)*
                    $($efield: $efield.try_done(stringify!($efield))?,)*
                    $($($ifield, $itable,)*)?
                    $any,
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
                _field: Option<::instant_xml::Id<'_>>,
                serializer: &mut ::instant_xml::Serializer<W>,
            ) -> ::std::result::Result<(), ::instant_xml::Error> {
                #[allow(unused_imports)]
                use ::instant_xml::ToXml;

                let mut context = ::instant_xml::ser::Context::<
                    { <[&str]>::len(&[$(stringify!($pfx)),*]) },
                >::default();
                context.default_ns = $ns;
                context.force_prefix = $force;
                context.prefixes = [$(::instant_xml::ser::Prefix { prefix: $pfx, ns: $pns }),*];
                let element = serializer.write_start($tag, $ns, Some(context))?;

                #[allow(unused_variables)]
                let declared: ::std::vec::Vec<(::std::string::String, ::std::string::String)> =
                    ::std::vec::Vec::new();
                $(
                    #[allow(unused_variables)]
                    let declared = $hook(self, serializer)?;
                )?

                $(
                    if ToXml::present(&self.$afield) {
                        serializer.write_attr($aname, $ans, &self.$afield)?;
                    }
                )*

                #[cfg(feature = "any")]
                let foreign_elements =
                    $crate::model::domain::any_extensions::AnySlot::slot_elements(&self.$any);
                #[cfg(feature = "any")]
                #[allow(unused_mut)]
                let mut scope = $crate::model::domain::any::write::write_foreign_attributes(
                    serializer,
                    &declared,
                    $ns,
                    $crate::model::domain::any_extensions::AnySlot::slot_attributes(&self.$any),
                    foreign_elements,
                )?;
                $($(
                    #[cfg(feature = "any")]
                    $crate::model::domain::any::write::declare_item_namespaces(
                        serializer,
                        &mut scope,
                        <$ity as $crate::model::domain::xml_item::XmlItem>::NS,
                        &self.$itable,
                        self.$ifield.len(),
                        <$ity as $crate::model::domain::xml_item::XmlItem>::TAG,
                    )?;
                )*)?
                #[cfg(feature = "any")]
                let has_foreign_elements = !foreign_elements.is_empty();
                #[cfg(not(feature = "any"))]
                let has_foreign_elements = false;

                // The derive always writes an open/close pair when the type has child element fields.
                // With `self_closing: true`, absent optional children (`None`) do not count.
                let self_closing = $crate::model::domain::xml_struct::xml_struct!(@self_closing $($self_closing)?);
                let has_elements = match self_closing {
                    true => false $(|| ToXml::present(&self.$efield))* $($(|| !self.$ifield.is_empty())*)?,
                    false => <[&str]>::len(&[$(stringify!($efield),)* $($(stringify!($ifield),)*)?]) > 0,
                };
                if !has_elements && !has_foreign_elements {
                    return serializer.end_empty();
                }

                serializer.end_start()?;
                $(
                    ToXml::serialize(
                        &self.$efield,
                        Some(::instant_xml::Id { ns: $ens, name: $ename }),
                        serializer,
                    )?;
                )*
                $($(
                    #[cfg(feature = "any")]
                    $crate::model::domain::xml_item::write_items(
                        serializer,
                        &self.$ifield,
                        &self.$itable,
                        &scope,
                    )?;
                    #[cfg(not(feature = "any"))]
                    $crate::model::domain::xml_item::write_items(
                        serializer,
                        &self.$ifield,
                        &self.$itable,
                    )?;
                )*)?
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

pub(crate) use xml_struct;
