#[cfg(any(
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read",
    feature = "io-lazy-read"
))]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;
    use std::io::Cursor;

    use threemf2::{
        core::{
            OptionalResourceId, OptionalResourceIndex,
            build::{Build, Item},
            mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
            model::{Model, Unit},
            object::{Object, ObjectKind, ObjectType},
            resources::Resources,
            slice::{
                MeshResolution, Polygon, Segment, Slice, SliceRef, SliceRefs, SliceStack,
                SliceVertex, SliceVertices, Slices,
            },
        },
        io::ThreemfPackage,
        io::content_types::{ContentTypes, DefaultContentTypeEnum, DefaultContentTypes},
        io::relationship::{Relationship, RelationshipType, Relationships},
    };

    /// Helper function to create a simple 3MF package with slice data
    fn create_test_model_with_slice() -> Model {
        // Create vertices for a simple mesh
        let vertices = Vertices {
            vertex: vec![
                Vertex::new(0.0, 0.0, 0.0),
                Vertex::new(10.0, 0.0, 0.0),
                Vertex::new(10.0, 10.0, 0.0),
                Vertex::new(0.0, 10.0, 0.0),
            ],
        };

        let triangles = Triangles {
            triangle: vec![
                Triangle {
                    v1: 0,
                    v2: 1,
                    v3: 2,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
                Triangle {
                    v1: 0,
                    v2: 2,
                    v3: 3,
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    p3: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                },
            ],
        };

        // Create slice stack with one slice
        let slice_vertices = SliceVertices {
            vertex: vec![
                SliceVertex { x: 0.0, y: 0.0 },
                SliceVertex { x: 10.0, y: 0.0 },
                SliceVertex { x: 10.0, y: 10.0 },
                SliceVertex { x: 0.0, y: 10.0 },
            ],
        };

        let slice = Slice {
            ztop: 0.1,
            vertices: Some(slice_vertices),
            polygon: vec![Polygon {
                startv: 0,
                segment: vec![
                    Segment {
                        v2: 1,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    },
                    Segment {
                        v2: 2,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    },
                    Segment {
                        v2: 3,
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    },
                ],
            }],
        };

        let slicestack = SliceStack {
            id: 1,
            zbottom: Some(0.0),
            slice: vec![slice],
            sliceref: vec![],
        };

        // Create object with mesh and slice reference
        let object = Object {
            id: 1,
            objecttype: Some(ObjectType::Model),
            thumbnail: None,
            partnumber: None,
            name: Some("TestObject".to_owned()),
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::new(1),
            slicepath: None,
            meshresolution: Some(MeshResolution::LowRes),
            kind: Some(ObjectKind::Mesh(Mesh {
                vertices,
                triangles,
                trianglesets: None,
                beamlattice: None,
            })),
        };

        // Create resources
        let resources = Resources {
            object: vec![object],
            basematerials: vec![],
            slicestack: vec![slicestack],
        };

        // Create build section
        let build = Build {
            uuid: None,
            item: vec![Item {
                objectid: 1,
                transform: None,
                partnumber: None,
                path: None,
                uuid: None,
            }],
        };

        Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: Some("s ".to_owned()),
            recommendedextensions: None,
            metadata: vec![],
            resources,
            build,
        }
    }

    #[cfg(feature = "io-write")]
    #[test]
    fn write_and_read_slice_model() {
        use threemf2::io::query::{get_objects, get_slice_stacks};

        // Create model with slice data
        let model = create_test_model_with_slice();

        // Write to buffer
        let mut buf = Cursor::new(Vec::new());
        let package = ThreemfPackage::new(
            model,
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
        package.write(&mut buf).expect("Error writing package");

        // Read back the package
        buf.set_position(0);
        let read_package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(&mut buf, false)
                .expect("Error reading package");

        // Verify slice stacks
        let slice_stacks: Vec<_> = get_slice_stacks(&read_package).collect();
        assert_eq!(slice_stacks.len(), 1);
        assert_eq!(slice_stacks[0].slicestack.id, 1);
        assert_eq!(slice_stacks[0].slicestack.zbottom, Some(0.0));

        // Verify object references slice stack
        let objects: Vec<_> = get_objects(&read_package).collect();
        assert_eq!(objects.len(), 1);
        let obj = &objects[0].object;
        assert!(obj.slicestackid.is_some());
        assert_eq!(obj.slicestackid.get(), Some(1));
        assert!(matches!(obj.meshresolution, Some(MeshResolution::LowRes)));

        // Verify namespace is present
        let ns = read_package.get_namespaces_on_model(None).unwrap();
        let has_slice_ns = ns
            .iter()
            .any(|n| n.uri == threemf2::threemf_namespaces::SLICE_NS);
        assert!(has_slice_ns, "Slice namespace should be present");
    }

    #[cfg(feature = "io-memory-optimized-read")]
    #[test]
    fn test_slice_stack_queries() {
        use threemf2::io::query::{get_slice_stack_from_model, get_slice_stacks_from_model};

        // Create model with slice data
        let model = create_test_model_with_slice();

        // Test get_slice_stacks_from_model
        let stacks: Vec<_> = get_slice_stacks_from_model(&model).collect();
        assert_eq!(stacks.len(), 1);
        assert_eq!(stacks[0].slicestack.id, 1);

        // Test get_slice_stack_from_model
        let stack = get_slice_stack_from_model(1, &model);
        assert!(stack.is_some());
        assert_eq!(stack.unwrap().slicestack.id, 1);

        let not_found = get_slice_stack_from_model(999, &model);
        assert!(not_found.is_none());
    }

    #[cfg(feature = "io-write")]
    #[test]
    fn test_sliceref_in_stack() {
        // Create a slice stack with sliceref instead of slices
        let slicestack = SliceStack {
            id: 1,
            zbottom: Some(0.0),
            slice: vec![],
            sliceref: vec![SliceRef {
                slicestackid: 2,
                slicepath: "/2D/slices.model".to_owned(),
            }],
        };

        let object = Object {
            id: 1,
            objecttype: Some(ObjectType::Model),
            thumbnail: None,
            partnumber: None,
            name: Some("SlicedObject".to_owned()),
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            uuid: None,
            slicestackid: OptionalResourceId::new(1),
            slicepath: None,
            meshresolution: Some(MeshResolution::FullRes),
            kind: Some(ObjectKind::Mesh(Mesh {
                vertices: Vertices { vertex: vec![] },
                triangles: Triangles { triangle: vec![] },
                trianglesets: None,
                beamlattice: None,
            })),
        };

        let model = Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: Some("s ".to_owned()),
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![object],
                basematerials: vec![],
                slicestack: vec![slicestack],
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
        };

        // Write and read back
        let mut buf = Cursor::new(Vec::new());
        let package = ThreemfPackage::new(
            model,
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
        package.write(&mut buf).expect("Error writing package");

        buf.set_position(0);
        let read_package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(&mut buf, false)
                .expect("Error reading package");

        use threemf2::io::query::get_slice_stacks;
        let stacks: Vec<_> = get_slice_stacks(&read_package).collect();
        assert_eq!(stacks.len(), 1);

        let stack = &stacks[0].slicestack;
        assert!(stack.slice.is_empty());
        assert!(!stack.sliceref.is_empty());

        assert_eq!(stack.sliceref.len(), 1);
        assert_eq!(stack.sliceref[0].slicestackid, 2);
        assert_eq!(stack.sliceref[0].slicepath, "/2D/slices.model");
    }
}
