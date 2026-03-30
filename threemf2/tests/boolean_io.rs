#[cfg(any(
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read",
    feature = "io-lazy-read"
))]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use std::{fs::File, path::PathBuf};

    #[cfg(feature = "io-memory-optimized-read")]
    #[test]
    fn read_threemf_package_memory_optimized() {
        use threemf2::io::ThreemfPackage;
        use threemf2::io::query::get_boolean_shape_objects;
        use threemf2::io::query::get_mesh_objects;

        let path = PathBuf::from("./tests/data/mesh-booleans-operations-material.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                // Verify we can get mesh objects (base geometry)
                let mesh_obj = get_mesh_objects(&package).collect::<Vec<_>>();
                assert_eq!(
                    mesh_obj.len(),
                    3,
                    "Expected 3 base mesh objects (cube, sphere, cylinder)"
                );

                // Verify we can find boolean shape objects
                let boolean_shapes = get_boolean_shape_objects(&package).collect::<Vec<_>>();
                assert_eq!(boolean_shapes.len(), 2, "Expected 2 boolean shapes");

                // Verify first boolean shape (Intersected - Object 6)
                let intersected = boolean_shapes.iter().find(|b| b.id == 6);
                assert!(intersected.is_some(), "Should find boolean shape with ID 6");
                if let Some(shape) = intersected {
                    assert!(
                        shape.is_intersection(),
                        "Object 6 should be Intersection operation"
                    );
                    assert_eq!(
                        shape.base_objectid(),
                        4,
                        "Object 6 base should be Object 4 (Cube)"
                    );
                    let operands: Vec<_> = shape.booleans().collect();
                    assert_eq!(operands.len(), 1, "Object 6 should have 1 operand");
                    assert_eq!(
                        operands[0].objectid, 5,
                        "Operand should reference Object 5 (Sphere)"
                    );
                }

                // Verify second boolean shape (Full part - Object 8)
                let full_part = boolean_shapes.iter().find(|b| b.id == 8);
                assert!(full_part.is_some(), "Should find boolean shape with ID 8");
                if let Some(shape) = full_part {
                    assert!(
                        shape.is_difference(),
                        "Object 8 should be Difference operation"
                    );
                    assert_eq!(
                        shape.base_objectid(),
                        6,
                        "Object 8 base should be Object 6 (nested boolean)"
                    );
                    let operands: Vec<_> = shape.booleans().collect();
                    assert_eq!(operands.len(), 3, "Object 8 should have 3 operands");
                    // All 3 operands should reference the cylinder (Object 3)
                    for operand in &operands {
                        assert_eq!(
                            operand.objectid, 3,
                            "Operand should reference Object 3 (Cylinder)"
                        );
                    }
                }

                // Verify namespaces include boolean operations
                let ns = package.get_namespaces_on_model(None).unwrap();
                let has_boolean_ns = ns
                    .iter()
                    .any(|n| n.uri == threemf2::threemf_namespaces::BOOLEAN_NS);
                assert!(
                    has_boolean_ns,
                    "Package should include Boolean Operations namespace"
                );
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }

    #[cfg(feature = "io-speed-optimized-read")]
    #[test]
    fn read_threemf_package_speed_optimized() {
        use threemf2::io::ThreemfPackage;
        use threemf2::io::query::get_boolean_shape_objects;
        use threemf2::io::query::get_mesh_objects;

        let path = PathBuf::from("./tests/data/mesh-booleans-operations-material.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_speed_optimized_deserializer(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                // Verify we can get mesh objects (base geometry)
                let mesh_obj = get_mesh_objects(&package).collect::<Vec<_>>();
                assert_eq!(
                    mesh_obj.len(),
                    3,
                    "Expected 3 base mesh objects (cube, sphere, cylinder)"
                );

                // Verify we can find boolean shape objects
                let boolean_shapes = get_boolean_shape_objects(&package).collect::<Vec<_>>();
                assert_eq!(boolean_shapes.len(), 2, "Expected 2 boolean shapes");

                // Verify first boolean shape (Intersected - Object 6)
                let intersected = boolean_shapes.iter().find(|b| b.id == 6);
                assert!(intersected.is_some(), "Should find boolean shape with ID 6");
                if let Some(shape) = intersected {
                    assert!(
                        shape.is_intersection(),
                        "Object 6 should be Intersection operation"
                    );
                    assert_eq!(
                        shape.base_objectid(),
                        4,
                        "Object 6 base should be Object 4 (Cube)"
                    );
                    let operands: Vec<_> = shape.booleans().collect();
                    assert_eq!(operands.len(), 1, "Object 6 should have 1 operand");
                }

                // Verify second boolean shape (Full part - Object 8)
                let full_part = boolean_shapes.iter().find(|b| b.id == 8);
                assert!(full_part.is_some(), "Should find boolean shape with ID 8");
                if let Some(shape) = full_part {
                    assert!(
                        shape.is_difference(),
                        "Object 8 should be Difference operation"
                    );
                    assert_eq!(
                        shape.base_objectid(),
                        6,
                        "Object 8 base should be Object 6 (nested boolean)"
                    );
                    let operands: Vec<_> = shape.booleans().collect();
                    assert_eq!(operands.len(), 3, "Object 8 should have 3 operands");
                }

                // Verify namespaces include boolean operations
                let ns = package.get_namespaces_on_model(None).unwrap();
                let has_boolean_ns = ns
                    .iter()
                    .any(|n| n.uri == threemf2::threemf_namespaces::BOOLEAN_NS);
                assert!(
                    has_boolean_ns,
                    "Package should include Boolean Operations namespace"
                );
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }

    #[cfg(all(feature = "io-lazy-read", feature = "io-memory-optimized-read"))]
    #[test]
    fn read_threemf_package_lazy_memory_optimized() {
        use threemf2::io::{CachePolicy, ThreemfPackageLazyReader};

        let path = PathBuf::from("./tests/data/mesh-booleans-operations-material.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
            reader,
            CachePolicy::NoCache,
        );

        assert!(result.is_ok());

        let mut namespaces = vec![];

        match result {
            Ok(package) => {
                let mut mesh_objects = 0;
                let mut boolean_shapes = 0;
                let mut intersected_found = false;
                let mut full_part_found = false;

                for model_path in package.model_paths() {
                    package
                        .with_model(model_path, |(model, ns)| {
                            use threemf2::io::query;

                            mesh_objects = query::get_mesh_objects_from_model(model).count();
                            boolean_shapes =
                                query::get_boolean_shape_objects_from_model(model).count();

                            // Check for specific boolean shapes
                            for boolean_ref in query::get_boolean_shape_objects_from_model(model) {
                                if boolean_ref.id == 6 && boolean_ref.is_intersection() {
                                    intersected_found = true;
                                    let operands: Vec<_> = boolean_ref.booleans().collect();
                                    assert_eq!(operands.len(), 1);
                                }
                                if boolean_ref.id == 8 && boolean_ref.is_difference() {
                                    full_part_found = true;
                                    let operands: Vec<_> = boolean_ref.booleans().collect();
                                    assert_eq!(operands.len(), 3);
                                }
                            }

                            namespaces.extend_from_slice(ns);
                        })
                        .unwrap();
                }

                // Verify counts
                assert_eq!(mesh_objects, 3, "Expected 3 mesh objects");
                assert_eq!(boolean_shapes, 2, "Expected 2 boolean shapes");
                assert!(
                    intersected_found,
                    "Should find Intersected boolean shape (ID 6)"
                );
                assert!(
                    full_part_found,
                    "Should find Full part boolean shape (ID 8)"
                );

                // Verify boolean namespace is present
                let has_boolean_ns = namespaces
                    .iter()
                    .any(|n| n.uri == threemf2::threemf_namespaces::BOOLEAN_NS);
                assert!(
                    has_boolean_ns,
                    "Package should include Boolean Operations namespace"
                );
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }

    #[cfg(all(feature = "io-lazy-read", feature = "io-speed-optimized-read"))]
    #[test]
    fn read_threemf_package_lazy_speed_optimized() {
        use threemf2::io::{CachePolicy, ThreemfPackageLazyReader};

        let path = PathBuf::from("./tests/data/mesh-booleans-operations-material.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackageLazyReader::from_reader_with_speed_optimized_deserializer(
            reader,
            CachePolicy::NoCache,
        );

        assert!(result.is_ok());

        let mut namespaces = vec![];

        match result {
            Ok(package) => {
                let mut mesh_objects = 0;
                let mut boolean_shapes = 0;
                let mut intersected_found = false;
                let mut full_part_found = false;

                for model_path in package.model_paths() {
                    package
                        .with_model(model_path, |(model, ns)| {
                            use threemf2::io::query;

                            mesh_objects = query::get_mesh_objects_from_model(model).count();
                            boolean_shapes =
                                query::get_boolean_shape_objects_from_model(model).count();

                            // Check for specific boolean shapes
                            for boolean_ref in query::get_boolean_shape_objects_from_model(model) {
                                if boolean_ref.id == 6 && boolean_ref.is_intersection() {
                                    intersected_found = true;
                                    let operands: Vec<_> = boolean_ref.booleans().collect();
                                    assert_eq!(operands.len(), 1);
                                }
                                if boolean_ref.id == 8 && boolean_ref.is_difference() {
                                    full_part_found = true;
                                    let operands: Vec<_> = boolean_ref.booleans().collect();
                                    assert_eq!(operands.len(), 3);
                                }
                            }

                            namespaces.extend_from_slice(ns);
                        })
                        .unwrap();
                }

                // Verify counts
                assert_eq!(mesh_objects, 3, "Expected 3 mesh objects");
                assert_eq!(boolean_shapes, 2, "Expected 2 boolean shapes");
                assert!(
                    intersected_found,
                    "Should find Intersected boolean shape (ID 6)"
                );
                assert!(
                    full_part_found,
                    "Should find Full part boolean shape (ID 8)"
                );

                // Verify boolean namespace is present
                let has_boolean_ns = namespaces
                    .iter()
                    .any(|n| n.uri == threemf2::threemf_namespaces::BOOLEAN_NS);
                assert!(
                    has_boolean_ns,
                    "Package should include Boolean Operations namespace"
                );
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }
}
