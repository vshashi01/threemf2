#[cfg(any(
    feature = "package-memory-optimized-read",
    feature = "io-speed-optimized-read",
    feature = "package-lazy-read"
))]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use threemf2::package::{
        ThreemfPackage,
        domain::validator::{ValidationRule, Validator},
        query::{get_mesh_objects, get_objects, get_slice_stacks},
    };

    use std::fs::File;
    use std::path::PathBuf;

    #[cfg(feature = "package-memory-optimized-read")]
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
                    .find(|stack_ref| stack_ref.origin_model_path.is_none())
                {
                    Some(root_stack_ref) => {
                        assert_eq!(root_stack_ref.view.sliceref_count(), 1);
                        let root_ref = root_stack_ref.view.slicerefs().next().unwrap();
                        match slice_stacks.iter().find(|stack_ref| {
                            stack_ref.origin_model_path == Some(root_ref.slicepath())
                                && stack_ref.view.id() == root_ref.slicestack_id()
                        }) {
                            Some(sub_stack) => {
                                assert!(sub_stack.view.has_owned_slices());
                                assert_eq!(sub_stack.view.slice_count(), 50);
                                for slice in sub_stack.view.slices() {
                                    if let Some(count) = slice.vertex_count() {
                                        assert!(count > 1);
                                        assert!(slice.polygon_count() > 0);
                                        for polygon in slice.polygons() {
                                            assert!(polygon.segment_count() > 1)
                                        }
                                    }
                                }
                            }
                            None => panic!(
                                "Couldn't find the appropriate stack with Id: {} in model path: {}",
                                root_ref.slicestack_id(),
                                root_ref.slicepath()
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
                    .find(|stack_ref| stack_ref.origin_model_path.is_none())
                {
                    Some(root_stack_ref) => {
                        assert_eq!(root_stack_ref.view.sliceref_count(), 1);

                        let root_ref = root_stack_ref.view.slicerefs().next().unwrap();
                        match slice_stacks.iter().find(|stack_ref| {
                            stack_ref.origin_model_path == Some(root_ref.slicepath())
                                && stack_ref.view.id() == root_ref.slicestack_id()
                        }) {
                            Some(sub_stack) => {
                                assert!(sub_stack.view.has_owned_slices());
                                assert_eq!(sub_stack.view.slice_count(), 50);
                                for slice in sub_stack.view.slices() {
                                    if let Some(count) = slice.vertex_count() {
                                        assert!(count > 1);
                                        assert!(slice.polygon_count() > 0);
                                        for polygon in slice.polygons() {
                                            assert!(polygon.segment_count() > 1)
                                        }
                                    }

                                    // ignore slices without vertices, usually the last slice
                                }
                            }
                            None => panic!(
                                "Couldn't find the appropriate stack with Id: {} in model path: {}",
                                root_ref.slicestack_id(),
                                root_ref.slicepath()
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
