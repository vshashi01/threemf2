//! End-to-end tests for foreign (xs:any / xs:anyAttribute) content with the `any` feature:
//! builder -> package write -> package read -> query.

#[cfg(all(
    feature = "any",
    feature = "package-read",
    feature = "package-write",
    feature = "package-lazy-read"
))]
mod tests {
    use pretty_assertions::assert_eq;

    use threemf2::{
        model::{
            builder::{ItemError, ModelBuilder, ModelError, ObjectId, Unit},
            domain::{
                any::{AnyElement, AnyNode},
                model::Model,
            },
            query::get_items_from_model,
        },
        package::{
            CachePolicy, ThreemfPackage, ThreemfPackageBuilder, ThreemfPackageLazyReader,
            query::get_items,
        },
    };

    use std::io::Cursor;

    const ACME_NS: &str = "http://acme.example/3mf/2026";

    fn add_triangle_object(builder: &mut ModelBuilder) -> ObjectId {
        builder
            .add_mesh_object(|obj| {
                obj.add_vertices(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
                    .add_triangle(&[0, 1, 2]);
                Ok(())
            })
            .expect("mesh object")
    }

    fn acme_info() -> AnyElement {
        AnyElement::new(ACME_NS, "info")
            .unwrap()
            .with_prefix_hint("acme")
            .with_attribute("", "rev", "3")
            .unwrap()
            .with_child(AnyElement::new(ACME_NS, "note").unwrap().with_text("hello"))
    }

    fn build_model() -> Model {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        let object = add_triangle_object(&mut builder);
        builder.add_build(None).unwrap();
        builder
            .add_build_item_advanced(object, |item| {
                item.partnumber("part-1")
                    .any_attribute(ACME_NS, "batch", "42")
                    .any_element(acme_info());
            })
            .unwrap();
        builder.add_build_item(object).unwrap();
        builder.build().expect("model")
    }

    fn write(model: Model) -> Cursor<Vec<u8>> {
        let mut package_builder = ThreemfPackageBuilder::new();
        package_builder.set_root_model(model);
        let package = package_builder.build().expect("package");

        let mut buf = Cursor::new(Vec::new());
        package.write(&mut buf).expect("write package");
        buf.set_position(0);
        buf
    }

    #[test]
    fn builder_adds_foreign_content_to_item() {
        let model = build_model();
        let items = get_items_from_model(&model).collect::<Vec<_>>();

        assert_eq!(items[0].any_attribute(ACME_NS, "batch"), Some("42"));
        assert_eq!(items[0].any_elements(), &[acme_info()]);
        assert!(items[1].any_extensions().is_empty());
    }

    #[test]
    fn foreign_content_survives_package_roundtrip() {
        let model = build_model();
        let mut buf = write(model.clone());

        let package = ThreemfPackage::from_reader(&mut buf, false).expect("read package");
        assert_eq!(package.root.build, model.build);

        let items = get_items(&package).collect::<Vec<_>>();
        assert_eq!(items.len(), 2);

        let first = &items[0].view;
        assert_eq!(first.part_number(), Some("part-1"));
        assert_eq!(first.any_attribute(ACME_NS, "batch"), Some("42"));
        assert_eq!(first.any_attributes().len(), 1);

        let info = first.any_elements_in(ACME_NS).collect::<Vec<_>>();
        assert_eq!(info.len(), 1);
        assert!(info[0].is(ACME_NS, "info"));
        assert_eq!(info[0].attribute("", "rev"), Some("3"));
        assert_eq!(info[0].child(ACME_NS, "note").unwrap().text(), "hello");
        assert!(matches!(
            info[0].children()[0],
            AnyNode::Element(ref note) if note.is(ACME_NS, "note")
        ));

        assert!(items[1].view.any_extensions().is_empty());
        assert_eq!(items[1].view.any_elements_in(ACME_NS).count(), 0);
    }

    #[test]
    fn foreign_content_is_read_by_lazy_reader() {
        let mut buf = write(build_model());

        let package = ThreemfPackageLazyReader::from_reader(&mut buf, CachePolicy::NoCache)
            .expect("read package");
        let batch = package
            .with_model(package.root_model_path(), |model| {
                get_items_from_model(model)
                    .next()
                    .and_then(|item| item.any_attribute(ACME_NS, "batch").map(str::to_owned))
            })
            .expect("root model");

        assert_eq!(batch.as_deref(), Some("42"));
    }

    #[test]
    fn builder_rejects_invalid_foreign_content() {
        let invalid: [fn(&mut threemf2::model::builder::ItemBuilder); 3] = [
            |item| {
                item.any_attribute("", "batch", "42");
            },
            |item| {
                item.any_attribute(ACME_NS, "not:valid", "42");
            },
            |item| {
                item.any_element(AnyElement::new("", "info").unwrap());
            },
        ];

        for add in invalid {
            let mut builder = ModelBuilder::new(Unit::Millimeter, true);
            let object = add_triangle_object(&mut builder);
            builder.add_build(None).unwrap();

            let result = builder.add_build_item_advanced(object, add);
            assert!(matches!(
                result,
                Err(ModelError::ItemError(ItemError::InvalidAnyExtension))
            ));
        }
    }
}
