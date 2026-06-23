#[cfg(any(
    feature = "package-memory-optimized-read",
    feature = "io-speed-optimized-read",
    feature = "package-lazy-read"
))]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use std::{fs::File, path::PathBuf};

    #[cfg(feature = "package-memory-optimized-read")]
    #[test]
    fn read_threemf_package_memory_optimized() {
        use threemf2::package::ThreemfPackage;
        use threemf2::package::query::get_components_objects;
        use threemf2::package::query::get_mesh_objects;
        use threemf2::package::query::get_objects;

        let path = PathBuf::from("./tests/data/mesh-composedpart-separate-model-files.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                assert_eq!(package.relationships.len(), 4);

                let objects = get_objects(&package).collect::<Vec<_>>();
                assert_eq!(objects.len(), 4);

                let mesh_objects = get_mesh_objects(&package).collect::<Vec<_>>();
                let can_find_object_by_uuid = mesh_objects.iter().find(|o| {
                    o.view.uuid().as_deref() == Some("79f98073-4eaa-4737-b065-041b98fb50a6")
                });
                assert_eq!(mesh_objects.len(), 3);
                assert!(can_find_object_by_uuid.is_some());

                let composedpart_objects = get_components_objects(&package).collect::<Vec<_>>();
                assert_eq!(composedpart_objects.len(), 1);

                let object_by_id = objects
                    .iter()
                    .filter(|r| matches!(r.origin_model_path, Some("/3D/Objects/Object.model")))
                    .find(|r| r.view.id() == 1);
                assert!(object_by_id.is_some());

                let can_find_build_item_by_uuid = package.root.build.item.iter().find(|i| {
                    i.uuid
                        .as_ref()
                        .map(|uuid| uuid.to_string().unwrap())
                        .as_deref()
                        == Some("637f47fa-39e6-4363-b3a9-100329fc5d9c")
                });
                assert!(can_find_build_item_by_uuid.is_some());
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }

    #[cfg(feature = "io-speed-optimized-read")]
    #[test]
    #[allow(deprecated)]
    fn read_threemf_package_speed_optimized() {
        use threemf2::package::ThreemfPackage;
        use threemf2::package::query::get_components_objects;
        use threemf2::package::query::get_mesh_objects;
        use threemf2::package::query::get_objects;

        let path = PathBuf::from("./tests/data/mesh-composedpart-separate-model-files.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_speed_optimized_deserializer(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                assert_eq!(package.relationships.len(), 4);

                let objects = get_objects(&package).collect::<Vec<_>>();
                assert_eq!(objects.len(), 4);

                let mesh_objects = get_mesh_objects(&package).collect::<Vec<_>>();
                let can_find_object_by_uuid = mesh_objects.iter().find(|o| {
                    o.view.uuid().as_deref() == Some("79f98073-4eaa-4737-b065-041b98fb50a6")
                });
                assert_eq!(mesh_objects.len(), 3);
                assert!(can_find_object_by_uuid.is_some());

                let composedpart_objects = get_components_objects(&package).collect::<Vec<_>>();
                assert_eq!(composedpart_objects.len(), 1);

                let object_by_id = objects
                    .iter()
                    .filter(|r| matches!(r.origin_model_path, Some("/3D/Objects/Object.model")))
                    .find(|r| r.view.id() == 1);
                assert!(object_by_id.is_some());

                let can_find_build_item_by_uuid = package.root.build.item.iter().find(|i| {
                    i.uuid
                        .as_ref()
                        .map(|uuid| uuid.to_string().unwrap())
                        .as_deref()
                        == Some("637f47fa-39e6-4363-b3a9-100329fc5d9c")
                });
                assert!(can_find_build_item_by_uuid.is_some());
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }

    #[cfg(all(
        feature = "package-lazy-read",
        feature = "package-memory-optimized-read"
    ))]
    #[test]
    fn read_threemf_package_lazy_memory_optimized() {
        use threemf2::package::{CachePolicy, ThreemfPackageLazyReader};

        let path = PathBuf::from("./tests/data/mesh-composedpart-separate-model-files.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
            reader,
            CachePolicy::NoCache,
        );

        assert!(result.is_ok());

        let mut namespaces = vec![];

        match result {
            Ok(package) => {
                assert_eq!(package.relationships().len(), 4);

                let mut total_model_paths = 0;
                let mut total_objects = 0;
                let mut mesh_objects = 0;
                let mut composedpart_objects = 0;

                // can find a specific mesh object with a specific uuid
                let mut found_object_by_uuid = false;

                // Check object by ID in specific model path
                let mut found_object_by_id = false;

                //iterate through all models and search for objects and the used namespaces
                for model_path in package.model_paths() {
                    total_model_paths += 1;
                    package
                        .with_model(model_path, |model| {
                            //check if some part with some id exists in a specific sub-model
                            if model_path.as_str() == "/3D/Objects/Object.model"
                                && model.resources.object.iter().any(|o| o.id == 1)
                            {
                                found_object_by_id = true;
                            }

                            for obj in &model.resources.object {
                                total_objects += 1;
                                if obj.get_mesh().is_some() {
                                    mesh_objects += 1;
                                    if obj
                                        .uuid
                                        .as_ref()
                                        .map(|uuid| uuid.to_string().unwrap())
                                        .as_deref()
                                        == Some("79f98073-4eaa-4737-b065-041b98fb50a6")
                                    {
                                        found_object_by_uuid = true;
                                    }
                                } else if obj.get_components_object().is_some() {
                                    composedpart_objects += 1;
                                }
                            }

                            namespaces.extend_from_slice(&model.used_namespaces());
                        })
                        .unwrap();
                }
                assert_eq!(total_model_paths, 3);
                assert_eq!(total_objects, 4);
                assert_eq!(mesh_objects, 3);
                assert_eq!(composedpart_objects, 1);
                assert!(found_object_by_uuid);
                assert!(found_object_by_id);

                // Check build item UUID in root model
                let root_model = package.root_model().unwrap();
                let can_find_build_item_by_uuid = root_model.build.item.iter().find(|i| {
                    i.uuid
                        .as_ref()
                        .map(|uuid| uuid.to_string().unwrap())
                        .as_deref()
                        == Some("637f47fa-39e6-4363-b3a9-100329fc5d9c")
                });
                assert!(can_find_build_item_by_uuid.is_some());

                //8 namespaces = (3 in each sub models + 2 in root model)
                assert_eq!(namespaces.len(), 8);
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }

    #[cfg(all(feature = "package-lazy-read", feature = "io-speed-optimized-read"))]
    #[test]
    #[allow(deprecated)]
    fn read_threemf_package_lazy_speed_optimized() {
        use threemf2::package::{CachePolicy, ThreemfPackageLazyReader};

        let path = PathBuf::from("./tests/data/mesh-composedpart-separate-model-files.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackageLazyReader::from_reader_with_speed_optimized_deserializer(
            reader,
            CachePolicy::NoCache,
        );

        assert!(result.is_ok());

        let mut namespaces = vec![];

        match result {
            Ok(package) => {
                assert_eq!(package.relationships().len(), 4);

                let mut total_model_paths = 0;
                let mut total_objects = 0;
                let mut mesh_objects = 0;
                let mut composedpart_objects = 0;

                // can find a specific mesh object with a specific uuid
                let mut found_object_by_uuid = false;

                // Check object by ID in specific model path
                let mut found_object_by_id = false;

                //iterate through all models and search for objects and the used namespaces
                for model_path in package.model_paths() {
                    total_model_paths += 1;
                    package
                        .with_model(model_path, |model| {
                            //check if some part with some id exists in a specific sub-model
                            if model_path.as_str() == "/3D/Objects/Object.model"
                                && model.resources.object.iter().any(|o| o.id == 1)
                            {
                                found_object_by_id = true;
                            }

                            for obj in &model.resources.object {
                                total_objects += 1;
                                if obj.get_mesh().is_some() {
                                    mesh_objects += 1;
                                    if obj
                                        .uuid
                                        .as_ref()
                                        .map(|uuid| uuid.to_string().unwrap())
                                        .as_deref()
                                        == Some("79f98073-4eaa-4737-b065-041b98fb50a6")
                                    {
                                        found_object_by_uuid = true;
                                    }
                                } else if obj.get_components_object().is_some() {
                                    composedpart_objects += 1;
                                }
                            }

                            namespaces.extend_from_slice(&model.used_namespaces());
                        })
                        .unwrap();
                }
                assert_eq!(total_model_paths, 3);
                assert_eq!(total_objects, 4);
                assert_eq!(mesh_objects, 3);
                assert_eq!(composedpart_objects, 1);
                assert!(found_object_by_uuid);
                assert!(found_object_by_id);

                // Check build item UUID in root model
                let root_model = package.root_model().unwrap();
                let can_find_build_item_by_uuid = root_model.build.item.iter().find(|i| {
                    i.uuid
                        .as_ref()
                        .map(|uuid| uuid.to_string().unwrap())
                        .as_deref()
                        == Some("637f47fa-39e6-4363-b3a9-100329fc5d9c")
                });
                assert!(can_find_build_item_by_uuid.is_some());

                //8 namespaces = (3 in each sub models + 2 in root model)
                assert_eq!(namespaces.len(), 8);
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }
}
