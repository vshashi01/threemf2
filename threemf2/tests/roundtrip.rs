#[cfg(all(
    any(
        feature = "package-memory-optimized-read",
        feature = "io-speed-optimized-read"
    ),
    feature = "package-write"
))]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use threemf2::{
        model::{
            OptionalResourceId,
            domain::{
                build::{Build, Item},
                mesh::{Mesh, Triangle, Triangles, Vertex, Vertices},
                model::{Model, ThreemfExtensions, Unit},
                object::{Object, ObjectKind, ObjectType},
                resources::Resources,
                types::OptionalResourceIndex,
            },
        },
        package::{ThreemfPackage, ThreemfPackageBuilder},
    };

    use std::io::Cursor;

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

        let model = Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: ThreemfExtensions::default(),
            recommendedextensions: ThreemfExtensions::default(),
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: Some("Mesh".into()),
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    uuid: None,
                    kind: Some(ObjectKind::Mesh(mesh.clone())),
                    slicestackid: OptionalResourceId::none(),
                    slicepath: None,
                    meshresolution: None,
                }],
                basematerials: vec![],
                slicestack: vec![],
                colorgroup: Vec::new(),
                texture2dgroup: Vec::new(),
                compositematerials: Vec::new(),
                multiproperties: Vec::new(),
                texture2d: Vec::new(),
                displacement2d: Vec::new(),
                normvectorgroup: Vec::new(),
                disp2dgroup: Vec::new(),
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

        let mut builder = ThreemfPackageBuilder::new();
        builder.set_root_model(model);
        let write_package = builder.build().expect("Error building package");

        let mut buf = Cursor::new(Vec::new());
        write_package
            .write(&mut buf)
            .expect("Error writing package");
        #[cfg(feature = "package-memory-optimized-read")]
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

        #[cfg(feature = "package-lazy-read")]
        {
            use threemf2::package::{CachePolicy, ThreemfPackageLazyReader};

            buf.set_position(0); // Reset cursor position
            let lazy_package =
                ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
                    &mut buf,
                    CachePolicy::NoCache,
                )
                .expect("Error reading package with lazy reader");

            // Verify basic structure
            assert_eq!(lazy_package.relationships().len(), 1);
            assert!(
                lazy_package
                    .root_model_path()
                    .as_str()
                    .contains("3Dmodel.model")
            );

            // Verify root model content
            let root_model = lazy_package.root_model().unwrap();
            assert_eq!(root_model.resources.object.len(), 1);
            assert_eq!(root_model.build.item.len(), 1);
            assert_eq!(root_model.used_namespaces().len(), 1);

            let obj = &root_model.resources.object[0];
            assert_eq!(obj.id, 1);
            assert_eq!(obj.name.as_deref(), Some("Mesh"));
            assert!(obj.get_mesh().is_some());
        }
    }
}
