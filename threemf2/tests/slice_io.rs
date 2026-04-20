#[cfg(any(
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read",
    feature = "io-lazy-read"
))]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use threemf2::io::{
        ThreemfPackage,
        query::{get_mesh_objects, get_objects, get_slice_stacks},
        validator::{ValidationRule, Validator},
    };

    use std::fs::File;
    use std::path::PathBuf;

    #[cfg(feature = "io-memory-optimized-read")]
    #[test]
    fn read_threemf_package_memory_optimized_single_slice_ref_with_multiple_slices() {
        let path = PathBuf::from("./tests/data/mesh-slice.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                assert_eq!(package.relationships.len(), 2);

                let objects = get_objects(&package).collect::<Vec<_>>();
                assert_eq!(objects.len(), 1);

                let mesh_objects = get_mesh_objects(&package).collect::<Vec<_>>();
                assert_eq!(mesh_objects.len(), 1);

                // println!("Package is {:?}", package.sub_models);
                let slice_stacks = get_slice_stacks(&package).collect::<Vec<_>>();
                assert_eq!(slice_stacks.len(), 2);

                match slice_stacks
                    .iter()
                    .find(|stack_ref| stack_ref.path.is_none())
                {
                    Some(root_stack_ref) => {
                        assert_eq!(root_stack_ref.slicestack.sliceref.len(), 1);
                        let root_ref = &root_stack_ref.slicestack.sliceref[0];
                        match slice_stacks.iter().find(|stack_ref| {
                            stack_ref.path == Some(&root_ref.slicepath)
                                && stack_ref.slicestack.id == root_ref.slicestackid
                        }) {
                            Some(sub_stack) => {
                                assert!(sub_stack.slicestack.has_owned_slices());
                                assert_eq!(sub_stack.slicestack.slice.len(), 50);
                                for slice in &sub_stack.slicestack.slice {
                                    if let Some(vertices) = &slice.vertices {
                                        assert!(vertices.vertex.len() > 1);
                                        assert!(!slice.polygon.is_empty());
                                        for polygon in &slice.polygon {
                                            assert!(polygon.segment.len() > 1)
                                        }
                                    }
                                }
                            }
                            None => panic!(
                                "Couldn't find the appropriate stack with Id: {} in model path: {}",
                                root_ref.slicestackid, root_ref.slicepath
                            ),
                        }
                    }
                    None => panic!("Root model stack not found"),
                }

                assert_eq!(1, package.root.build.item.len());

                let ns = package.get_namespaces_on_model(None).unwrap();
                assert_eq!(ns.len(), 2);

                let validator = Validator::new()
                    .with_rule(ValidationRule::ObjectIdReference)
                    .with_rule(ValidationRule::BaseMaterialReference)
                    .with_rule(ValidationRule::BuildItemReference)
                    .with_rule(ValidationRule::ComponentReference);

                let result = validator.validate_package(&package);
                assert!(result.is_valid);
                assert_eq!(result.issues.len(), 0);
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }

    #[cfg(feature = "io-speed-optimized-read")]
    #[test]
    fn read_threemf_package_speed_optimized_single_slice_ref_with_multiple_slices() {
        let path = PathBuf::from("./tests/data/mesh-slice.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_speed_optimized_deserializer(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                assert_eq!(package.relationships.len(), 2);

                let objects = get_objects(&package).collect::<Vec<_>>();
                assert_eq!(objects.len(), 1);

                let mesh_objects = get_mesh_objects(&package).collect::<Vec<_>>();
                assert_eq!(mesh_objects.len(), 1);

                // println!("Package is {:?}", package.sub_models);
                let slice_stacks = get_slice_stacks(&package).collect::<Vec<_>>();
                assert_eq!(slice_stacks.len(), 2);

                match slice_stacks
                    .iter()
                    .find(|stack_ref| stack_ref.path.is_none())
                {
                    Some(root_stack_ref) => {
                        assert_eq!(root_stack_ref.slicestack.sliceref.len(), 1);

                        let root_ref = &root_stack_ref.slicestack.sliceref[0];
                        match slice_stacks.iter().find(|stack_ref| {
                            stack_ref.path == Some(&root_ref.slicepath)
                                && stack_ref.slicestack.id == root_ref.slicestackid
                        }) {
                            Some(sub_stack) => {
                                assert!(sub_stack.slicestack.has_owned_slices());
                                assert_eq!(sub_stack.slicestack.slice.len(), 50);
                                for slice in &sub_stack.slicestack.slice {
                                    if let Some(vertices) = &slice.vertices {
                                        assert!(vertices.vertex.len() > 1);
                                        assert!(!slice.polygon.is_empty());
                                        for polygon in &slice.polygon {
                                            assert!(polygon.segment.len() > 1)
                                        }
                                    }

                                    // ignore slices without vertices, usually the last slice
                                }
                            }
                            None => panic!(
                                "Couldn't find the appropriate stack with Id: {} in model path: {}",
                                root_ref.slicestackid, root_ref.slicepath
                            ),
                        }
                    }
                    None => panic!("Root model stack not found"),
                }

                assert_eq!(1, package.root.build.item.len());

                let ns = package.get_namespaces_on_model(None).unwrap();
                assert_eq!(ns.len(), 2);

                let validator = Validator::new()
                    .with_rule(ValidationRule::ObjectIdReference)
                    .with_rule(ValidationRule::BaseMaterialReference)
                    .with_rule(ValidationRule::BuildItemReference)
                    .with_rule(ValidationRule::ComponentReference);

                let result = validator.validate_package(&package);
                assert!(result.is_valid);
                assert_eq!(result.issues.len(), 0);
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }
}
