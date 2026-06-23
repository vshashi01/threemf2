#[cfg(any(
    feature = "package-memory-optimized-read",
    feature = "io-speed-optimized-read",
))]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use std::{fs::File, path::PathBuf};

    use threemf2::model::Color;
    use threemf2::model::query::get_color_groups_from_model;
    use threemf2::package::query::get_mesh_objects;

    #[cfg(feature = "package-memory-optimized-read")]
    #[test]
    fn read_threemf_package_memory_optimized() {
        use threemf2::package::ThreemfPackage;

        let path = PathBuf::from("./tests/data/mesh_vertexcolor-material.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                // Verify 2 mesh objects

                use threemf2::threemf_namespaces::ThreemfNamespace;
                let mesh_objects = get_mesh_objects(&package).collect::<Vec<_>>();
                assert_eq!(mesh_objects.len(), 1);

                // Verify 2 color groups
                let color_groups: Vec<_> = get_color_groups_from_model(&package.root).collect();
                assert_eq!(color_groups.len(), 1);

                // Verify first color group (id=2) with 4 colors
                let first_group = color_groups.iter().find(|cg| cg.id() == 2).unwrap();
                assert_eq!(first_group.color_count(), 4);

                // Verify color values using Color type
                let expected_colors = [
                    Color::from_hex("#FF0000FF").unwrap(), // Red
                    Color::from_hex("#0000FFFF").unwrap(), // Blue
                    Color::from_hex("#00FF00FF").unwrap(), // Green
                    Color::from_hex("#FFFFFFFF").unwrap(), // White
                ];
                (0..first_group.color_count()).for_each(|i| {
                    assert_eq!(first_group.color_at(i).unwrap(), expected_colors[i]);
                });

                // Verify material namespace is present
                let ns = package.get_namespaces_on_model(None).unwrap();
                assert!(ns.iter().any(|n| matches!(n, ThreemfNamespace::Material)));
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

        let path = PathBuf::from("./tests/data/mesh_vertexcolor-material.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_speed_optimized_deserializer(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                // Verify 2 mesh objects

                use threemf2::threemf_namespaces::ThreemfNamespace;
                let mesh_objects = get_mesh_objects(&package).collect::<Vec<_>>();
                assert_eq!(mesh_objects.len(), 1);

                // Verify 2 color groups
                let color_groups: Vec<_> = get_color_groups_from_model(&package.root).collect();
                assert_eq!(color_groups.len(), 1);

                // Verify first color group (id=2) with 4 colors
                let first_group = color_groups.iter().find(|cg| cg.id() == 2).unwrap();
                assert_eq!(first_group.color_count(), 4);

                // Verify color values using Color type
                let expected_colors = [
                    Color::from_hex("#FF0000FF").unwrap(), // Red
                    Color::from_hex("#0000FFFF").unwrap(), // Blue
                    Color::from_hex("#00FF00FF").unwrap(), // Green
                    Color::from_hex("#FFFFFFFF").unwrap(), // White
                ];
                (0..first_group.color_count()).for_each(|i| {
                    assert_eq!(first_group.color_at(i).unwrap(), expected_colors[i]);
                });

                // Verify material namespace is present
                let ns = package.get_namespaces_on_model(None).unwrap();
                assert!(ns.iter().any(|n| matches!(n, ThreemfNamespace::Material)));
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }
}
