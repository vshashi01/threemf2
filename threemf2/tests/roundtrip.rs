#[cfg(all(
    any(
        feature = "io-memory-optimized-read",
        feature = "io-speed-optimized-read"
    ),
    feature = "io-write"
))]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use threemf2::{
        core::{
            OptionalResourceId,
            build::{Build, Item},
            mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
            model::{Model, Unit},
            object::{Object, ObjectType},
            object_kind::ObjectKind,
            resources::Resources,
            types::OptionalResourceIndex,
        },
        io::{
            ThreemfPackage,
            content_types::{ContentTypes, DefaultContentTypeEnum, DefaultContentTypes},
            relationship::{Relationship, RelationshipType, Relationships},
        },
    };

    use std::{collections::HashMap, io::Cursor};

    #[test]
    fn roundtrip_threemfpackage_test() {
        let vertices = Vertices {
            vertex: vec![
                Vertex::new(0.0, 0.0, 0.0),
                Vertex::new(0.0, 2.0, 0.0),
                Vertex::new(0.0, 1.0, 1.0),
            ],
        };

        let triangles = Triangles {
            triangle: vec![Triangle {
                v1: 0,
                v2: 1,
                v3: 2,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            }],
        };

        let mesh = Mesh {
            triangles,
            vertices,
            trianglesets: None,
            beamlattice: None,
        };

        let write_package = ThreemfPackage::new(
            Model {
                unit: Some(Unit::Millimeter),
                requiredextensions: None,
                recommendedextensions: None,
                metadata: vec![],
                resources: Resources {
                    object: vec![Object {
                        id: 1,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("Mesh".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: Some(ObjectKind::Mesh(mesh.clone())), // mesh: Some(mesh.clone()),
                                                                    // components: None,
                                                                    // booleanshape: None,
                    }],
                    basematerials: vec![],
                },
                build: Build {
                    uuid: None,
                    item: vec![Item {
                        objectid: 1,
                        transform: None,
                        partnumber: None,
                        path: None,
                        uuid: None,
                    }],
                },
            },
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::from([(
                "_rels/.rels".to_owned(),
                Relationships {
                    relationships: vec![Relationship {
                        id: "rel0".to_owned(),
                        target: "3D/3Dmodel.model".to_owned(),
                        relationship_type: RelationshipType::Model,
                    }],
                },
            )]),
            ContentTypes {
                defaults: vec![
                    DefaultContentTypes {
                        extension: "rels".to_owned(),
                        content_type: DefaultContentTypeEnum::Relationship,
                    },
                    DefaultContentTypes {
                        extension: "model".to_owned(),
                        content_type: DefaultContentTypeEnum::Model,
                    },
                ],
            },
        );

        let mut buf = Cursor::new(Vec::new());

        write_package
            .write(&mut buf)
            .expect("Error writing package");
        #[cfg(feature = "io-memory-optimized-read")]
        {
            let package =
                ThreemfPackage::from_reader_with_memory_optimized_deserializer(&mut buf, false)
                    .expect("Error reading package");
            assert_eq!(package, write_package);

            let ns = package.get_namespaces_on_model(None).unwrap();
            assert_eq!(ns.len(), 1);
        }
        #[cfg(feature = "io-speed-optimized-read")]
        {
            let package =
                ThreemfPackage::from_reader_with_speed_optimized_deserializer(&mut buf, false)
                    .expect("Error reading package");
            assert_eq!(package, write_package);

            let ns = package.get_namespaces_on_model(None).unwrap();
            assert_eq!(ns.len(), 1);
        }

        #[cfg(feature = "io-lazy-read")]
        {
            use threemf2::io::{CachePolicy, ThreemfPackageLazyReader};

            buf.set_position(0); // Reset cursor position
            let lazy_package =
                ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
                    &mut buf,
                    CachePolicy::NoCache,
                )
                .expect("Error reading package with lazy reader");

            // Verify basic structure
            assert_eq!(lazy_package.relationships().len(), 1);
            assert!(lazy_package.root_model_path().contains("3Dmodel.model"));

            // Verify root model content
            let (root_model, ns) = lazy_package.root_model().unwrap();
            assert_eq!(root_model.resources.object.len(), 1);
            assert_eq!(root_model.build.item.len(), 1);
            assert_eq!(ns.len(), 1);

            let obj = &root_model.resources.object[0];
            assert_eq!(obj.id, 1);
            assert_eq!(obj.name, Some("Mesh".to_owned()));
            assert!(obj.kind.as_ref().unwrap().get_mesh().is_some());
        }
    }
}
