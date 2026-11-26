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
            builder::{ItemBuilder, ItemError, ModelBuilder, ModelError, ObjectId, Unit},
            domain::{
                any::{AnyElement, AnyError, AnyNode},
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
                    .any_attribute(ACME_NS, "batch", "42")?
                    .any_element(acme_info())?;
                Ok(())
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
        type AddItem = fn(&mut ItemBuilder) -> Result<(), ItemError>;
        let invalid: [(AddItem, AnyError); 3] = [
            (
                |item| {
                    item.any_attribute("", "batch", "42")?;
                    Ok(())
                },
                AnyError::NotForeignNamespace(String::new()),
            ),
            (
                |item| {
                    item.any_attribute(ACME_NS, "not:valid", "42")?;
                    Ok(())
                },
                AnyError::InvalidName("not:valid".to_owned()),
            ),
            (
                |item| {
                    item.any_element(AnyElement::new("", "info").unwrap())?;
                    Ok(())
                },
                AnyError::NotForeignNamespace(String::new()),
            ),
        ];

        for (add, expected) in invalid {
            let mut builder = ModelBuilder::new(Unit::Millimeter, true);
            let object = add_triangle_object(&mut builder);
            builder.add_build(None).unwrap();

            let result = builder.add_build_item_advanced(object, add);
            match result {
                Err(ModelError::ItemError(ItemError::InvalidAnyExtension(err))) => {
                    assert_eq!(err, expected)
                }
                other => panic!("unexpected result: {:?}", other.map(|_| ())),
            }
        }
    }

    #[test]
    fn builder_rejects_invalid_item_path() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        let object = add_triangle_object(&mut builder);
        builder.add_build(None).unwrap();

        let result = builder.add_build_item_advanced(object, |item| {
            item.path("")?;
            Ok(())
        });
        assert!(matches!(
            result,
            Err(ModelError::ItemError(ItemError::InvalidPath(_)))
        ));
    }
    /// A model with foreign content in every core extension point, plus an unknown required
    /// extension.
    fn model_xml_with_foreign_content() -> String {
        let core = threemf2::threemf_namespaces::CORE_NS;
        format!(
            r##"<model xmlns="{core}" xmlns:acme="{ACME_NS}" unit="millimeter" requiredextensions="acme" acme:model="1">
  <metadata name="Title" acme:meta="2">Hello</metadata>
  <resources acme:res="3">
    <basematerials id="1" acme:bm="4"><base name="Red" displaycolor="#FF0000" acme:base="5"/></basematerials>
    <object id="2" type="model" acme:obj="6">
      <mesh acme:mesh="7">
        <vertices acme:verts="8"><vertex x="0" y="0" z="0"/><vertex x="1" y="0" z="0"/><vertex x="0" y="1" z="0"/></vertices>
        <triangles acme:tris="9"><triangle v1="0" v2="1" v3="2"/></triangles>
        <acme:meshdata/>
      </mesh>
      <acme:objdata/>
    </object>
    <object id="3" type="model"><acme:shape kind="sphere"/></object>
    <object id="4" type="model"><components acme:comps="10"><component objectid="2" acme:comp="11"><acme:compdata/></component></components></object>
    <acme:texture id="20"/>
  </resources>
  <build acme:build="12"><item objectid="4" acme:item="13"><acme:itemdata/></item></build>
  <acme:modeldata/>
</model>"##
        )
    }

    fn assert_foreign_content(model: &Model) {
        use threemf2::model::domain::object::ObjectKind;

        let attr = |ext: &threemf2::model::domain::any::AnyAttributes, name: &str| {
            ext.get(ACME_NS, name).map(str::to_owned)
        };
        let names = |elements: &[AnyElement]| {
            elements
                .iter()
                .map(|e| e.name().to_owned())
                .collect::<Vec<_>>()
        };

        assert_eq!(
            attr(model.any_extensions.attributes(), "model").as_deref(),
            Some("1")
        );
        assert_eq!(names(model.any_extensions.elements()), ["modeldata"]);
        assert_eq!(
            attr(model.metadata[0].any_extensions.attributes(), "meta").as_deref(),
            Some("2")
        );
        assert_eq!(model.metadata[0].value.as_deref(), Some("Hello"));

        let resources = &model.resources;
        assert_eq!(
            attr(resources.any_extensions.attributes(), "res").as_deref(),
            Some("3")
        );
        assert_eq!(names(resources.any_extensions.elements()), ["texture"]);
        let bm = &resources.basematerials[0];
        assert_eq!(
            attr(bm.any_extensions.attributes(), "bm").as_deref(),
            Some("4")
        );
        assert_eq!(
            attr(bm.base[0].any_extensions.attributes(), "base").as_deref(),
            Some("5")
        );

        let mesh_object = &resources.object[0];
        assert_eq!(
            attr(mesh_object.any_extensions.attributes(), "obj").as_deref(),
            Some("6")
        );
        assert_eq!(names(mesh_object.any_extensions.elements()), ["objdata"]);
        let mesh = mesh_object.get_mesh().expect("mesh");
        assert_eq!(
            attr(mesh.any_extensions.attributes(), "mesh").as_deref(),
            Some("7")
        );
        assert_eq!(names(mesh.any_extensions.elements()), ["meshdata"]);
        assert_eq!(
            attr(mesh.vertices.any_extensions.attributes(), "verts").as_deref(),
            Some("8")
        );
        assert_eq!(
            attr(mesh.triangles.any_extensions.attributes(), "tris").as_deref(),
            Some("9")
        );
        assert_eq!(mesh.vertices.vertex.len(), 3);

        match &resources.object[1].kind {
            Some(ObjectKind::Any(shape)) => {
                assert!(shape.is(ACME_NS, "shape"));
                assert_eq!(shape.attribute("", "kind"), Some("sphere"));
            }
            other => panic!("expected a foreign shape, got {other:?}"),
        }
        assert!(resources.object[1].any_extensions.is_empty());

        let components = resources.object[2]
            .get_components_object()
            .expect("components");
        assert_eq!(
            attr(components.any_extensions.attributes(), "comps").as_deref(),
            Some("10")
        );
        let component = &components.component[0];
        assert_eq!(
            attr(component.any_extensions.attributes(), "comp").as_deref(),
            Some("11")
        );
        assert_eq!(names(component.any_extensions.elements()), ["compdata"]);

        assert_eq!(
            attr(model.build.any_extensions.attributes(), "build").as_deref(),
            Some("12")
        );
        let item = &model.build.item[0];
        assert_eq!(
            attr(item.any_extensions.attributes(), "item").as_deref(),
            Some("13")
        );
        assert_eq!(names(item.any_extensions.elements()), ["itemdata"]);
    }

    #[test]
    fn foreign_content_in_all_core_elements_roundtrips() {
        let xml = model_xml_with_foreign_content();
        let model = instant_xml::from_str::<Model>(&xml).expect("read model");
        assert_foreign_content(&model);

        // The unknown required extension and every foreign namespace survive the package writer,
        // which strips unused namespace declarations from <model>.
        let mut buf = write(model.clone());
        let package = ThreemfPackage::from_reader(&mut buf, false).expect("read package");
        assert_foreign_content(&package.root);
        assert_eq!(package.root, model);
    }

    #[test]
    fn builder_and_views_for_objects_components_and_model() {
        use threemf2::model::query::{
            get_components_objects_from_model, get_mesh_objects_from_model, get_model_view,
        };

        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        builder.any_attribute(ACME_NS, "model", "1").unwrap();
        builder
            .any_element(AnyElement::new(ACME_NS, "modeldata").unwrap())
            .unwrap();

        let mesh_id = builder
            .add_mesh_object(|obj| {
                obj.any_attribute(ACME_NS, "obj", "2")?
                    .any_element(acme_info())?;
                obj.add_vertices(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
                    .add_triangle(&[0, 1, 2]);
                Ok(())
            })
            .unwrap();
        let assembly_id = builder
            .add_components_object(|obj| {
                obj.add_component_advanced(mesh_id, |component| {
                    component.any_attribute(ACME_NS, "comp", "3")?;
                    Ok(())
                })?;
                Ok(())
            })
            .unwrap();
        builder.add_build(None).unwrap();
        builder.add_build_item(assembly_id).unwrap();

        let invalid = builder.add_mesh_object(|obj| {
            obj.any_attribute("", "obj", "2")?;
            Ok(())
        });
        assert!(matches!(
            invalid,
            Err(threemf2::model::builder::MeshObjectError::InvalidAnyExtension(_))
        ));

        let model = builder.build().unwrap();
        let mut buf = write(model);
        let package = ThreemfPackage::from_reader(&mut buf, false).expect("read package");
        let model = &package.root;

        let view = get_model_view(model);
        assert_eq!(view.any_attribute(ACME_NS, "model"), Some("1"));
        assert_eq!(view.any_elements_in(ACME_NS).count(), 1);

        let mesh_object = get_mesh_objects_from_model(model).next().unwrap();
        assert_eq!(mesh_object.any_attribute(ACME_NS, "obj"), Some("2"));
        assert_eq!(mesh_object.any_elements(), &[acme_info()]);
        assert!(mesh_object.mesh_any_extensions().is_empty());

        let assembly = get_components_objects_from_model(model).next().unwrap();
        let component = assembly.components().next().unwrap();
        assert_eq!(component.any_attribute(ACME_NS, "comp"), Some("3"));
    }
}
