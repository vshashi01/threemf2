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
        use threemf2::io::query::get_components_objects;
        use threemf2::io::query::get_mesh_objects;
        use threemf2::io::query::get_objects;

        let path = PathBuf::from("./tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, false);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                use threemf2::io::validator::{ValidationRule, Validator};

                assert_eq!(package.relationships.len(), 1);

                let objects = get_objects(&package).collect::<Vec<_>>();
                assert_eq!(objects.len(), 4);

                let mesh_objects = get_mesh_objects(&package).collect::<Vec<_>>();
                assert_eq!(mesh_objects.len(), 3);

                let composedpart_objects = get_components_objects(&package).collect::<Vec<_>>();
                assert_eq!(composedpart_objects.len(), 1);

                // let object_by_id = get_object_ref_from_id(1, &package, None, None);
                let object_by_id = objects
                    .iter()
                    .filter(|r| r.path.is_none())
                    .find(|r| r.object.id == 1);
                assert!(object_by_id.is_some());

                assert_eq!(2, package.root.build.item.len());

                let ns = package.get_namespaces_on_model(None).unwrap();
                assert_eq!(ns.len(), 3);

                let validator = Validator::new()
                    .with_rule(ValidationRule::ObjectIdReference)
                    .with_rule(ValidationRule::BaseMaterialReference)
                    .with_rule(ValidationRule::BuildItemReference)
                    .with_rule(ValidationRule::ComponentReference);

                let result = validator.validate_package(&package);
                assert!(!result.is_valid);
                assert_eq!(result.issues.len(), 3);
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
        use threemf2::io::query::get_components_objects;
        use threemf2::io::query::get_mesh_objects;
        use threemf2::io::query::get_objects;

        let path = PathBuf::from("./tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_speed_optimized_deserializer(reader, false);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                use threemf2::io::validator::{ValidationRule, Validator};

                assert_eq!(package.relationships.len(), 1);

                let objects = get_objects(&package).collect::<Vec<_>>();
                assert_eq!(objects.len(), 4);

                let mesh_objects = get_mesh_objects(&package).collect::<Vec<_>>();
                assert_eq!(mesh_objects.len(), 3);

                let composedpart_objects = get_components_objects(&package).collect::<Vec<_>>();
                assert_eq!(composedpart_objects.len(), 1);

                // let object_by_id = get_object_ref_from_id(1, &package, None, None);
                let object_by_id = objects
                    .iter()
                    .filter(|r| r.path.is_none())
                    .find(|r| r.object.id == 1);
                assert!(object_by_id.is_some());

                assert_eq!(2, package.root.build.item.len());

                let ns = package.get_namespaces_on_model(None).unwrap();
                assert_eq!(ns.len(), 3);

                let validator = Validator::new()
                    .with_rule(ValidationRule::ObjectIdReference)
                    .with_rule(ValidationRule::BaseMaterialReference)
                    .with_rule(ValidationRule::BuildItemReference)
                    .with_rule(ValidationRule::ComponentReference);

                let result = validator.validate_package(&package);
                assert!(!result.is_valid);
                assert_eq!(result.issues.len(), 3);
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

        let path = PathBuf::from("./tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
            reader,
            CachePolicy::NoCache,
        );

        assert!(result.is_ok());

        let mut namespaces = vec![];

        match result {
            Ok(package) => {
                assert_eq!(package.relationships().len(), 1);

                let mut total_model_paths = 0;
                let mut total_objects = 0;
                let mut mesh_objects = 0;
                let mut composedpart_objects = 0;

                // Check object by ID in specific model path
                let mut found_object_by_id = false;

                //iterate through all models and search for objects and the used namespaces
                for model_path in package.model_paths() {
                    total_model_paths += 1;
                    package
                        .with_model(model_path, |(model, ns)| {
                            //check if some part with some specific id exists
                            if model.resources.object.iter().any(|o| o.id == 1) {
                                found_object_by_id = true;
                            }

                            for obj in &model.resources.object {
                                total_objects += 1;
                                if obj.get_mesh().is_some() {
                                    mesh_objects += 1;
                                } else if obj.get_components_object().is_some() {
                                    composedpart_objects += 1;
                                }
                            }

                            namespaces.extend_from_slice(ns);
                        })
                        .unwrap();
                }
                assert_eq!(total_model_paths, 1);
                assert_eq!(total_objects, 4);
                assert_eq!(mesh_objects, 3);
                assert_eq!(composedpart_objects, 1);
                assert!(found_object_by_id);

                //contains core, prod and material
                assert_eq!(namespaces.len(), 3);
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

        let path = PathBuf::from("./tests/data/mesh-composedpart.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackageLazyReader::from_reader_with_speed_optimized_deserializer(
            reader,
            CachePolicy::NoCache,
        );

        assert!(result.is_ok());

        let mut namespaces = vec![];

        match result {
            Ok(package) => {
                assert_eq!(package.relationships().len(), 1);

                let mut total_model_paths = 0;
                let mut total_objects = 0;
                let mut mesh_objects = 0;
                let mut composedpart_objects = 0;

                // Check object by ID in specific model path
                let mut found_object_by_id = false;

                //iterate through all models and search for objects and the used namespaces
                for model_path in package.model_paths() {
                    total_model_paths += 1;
                    package
                        .with_model(model_path, |(model, ns)| {
                            //check if some part with some specific id exists
                            if model.resources.object.iter().any(|o| o.id == 1) {
                                found_object_by_id = true;
                            }

                            for obj in &model.resources.object {
                                total_objects += 1;
                                if obj.get_mesh().is_some() {
                                    mesh_objects += 1;
                                } else if obj.get_components_object().is_some() {
                                    composedpart_objects += 1;
                                }
                            }

                            namespaces.extend_from_slice(ns);
                        })
                        .unwrap();
                }
                assert_eq!(total_model_paths, 1);
                assert_eq!(total_objects, 4);
                assert_eq!(mesh_objects, 3);
                assert_eq!(composedpart_objects, 1);
                assert!(found_object_by_id);

                //contains core, prod and material
                assert_eq!(namespaces.len(), 3);
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }
}
