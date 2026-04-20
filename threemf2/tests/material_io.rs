#[cfg(any(
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read",
))]
#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use std::{fs::File, path::PathBuf};

    use threemf2::core::Color;
    use threemf2::io::query::{get_color_groups_from_model, get_mesh_objects};

    #[cfg(feature = "io-memory-optimized-read")]
    #[test]
    fn read_threemf_package_memory_optimized() {
        use threemf2::io::ThreemfPackage;

        let path = PathBuf::from("./tests/data/mesh_vertexcolor-material.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                // Verify 2 mesh objects
                let mesh_objects = get_mesh_objects(&package).collect::<Vec<_>>();
                assert_eq!(mesh_objects.len(), 1);

                // Verify 2 color groups
                let color_groups: Vec<_> = get_color_groups_from_model(&package.root).collect();
                assert_eq!(color_groups.len(), 1);

                // Verify first color group (id=2) with 4 colors
                let first_group = color_groups
                    .iter()
                    .find(|cg| cg.colorgroup.id == 2)
                    .unwrap();
                assert_eq!(first_group.colorgroup.color.len(), 4);

                // Verify color values using Color type
                let expected_colors = [
                    Color::from_hex("#FF0000FF").unwrap(), // Red
                    Color::from_hex("#0000FFFF").unwrap(), // Blue
                    Color::from_hex("#00FF00FF").unwrap(), // Green
                    Color::from_hex("#FFFFFFFF").unwrap(), // White
                ];
                for (i, color_element) in first_group.colorgroup.color.iter().enumerate() {
                    assert_eq!(color_element.color, expected_colors[i]);
                }

                // Verify triangles with vertex colors point to valid color resources
                let mesh = &mesh_objects[0].mesh();
                let triangles_with_color: Vec<_> = mesh
                    .triangles
                    .triangle
                    .iter()
                    .filter(|t| {
                        t.pid.is_some() && (t.p1.is_some() || t.p2.is_some() || t.p3.is_some())
                    })
                    .collect();
                assert!(
                    !triangles_with_color.is_empty(),
                    "No triangles with vertex colors found"
                );

                // Verify each triangle with colors points to the color group
                for triangle in &triangles_with_color {
                    let pid = triangle.pid.get().unwrap();
                    // Check at least one of p1, p2, p3 points to a valid color index
                    let has_valid_index = [triangle.p1, triangle.p2, triangle.p3]
                        .iter()
                        .filter_map(|p| p.get())
                        .any(|pindex| {
                            let pindex_usize = pindex as usize;
                            pindex_usize < first_group.colorgroup.color.len()
                        });
                    assert_eq!(pid, 2, "Triangle pid should reference color group 2");
                    assert!(
                        has_valid_index,
                        "Triangle should have at least one valid color index"
                    );
                }

                // Verify material namespace is present
                let ns = package.get_namespaces_on_model(None).unwrap();
                assert!(ns.iter().any(|n| n.uri.contains("material")));
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

        let path = PathBuf::from("./tests/data/mesh_vertexcolor-material.3mf");
        let reader = File::open(path).unwrap();

        let result = ThreemfPackage::from_reader_with_speed_optimized_deserializer(reader, true);

        assert!(result.is_ok());

        match result {
            Ok(package) => {
                // Verify 2 mesh objects
                let mesh_objects = get_mesh_objects(&package).collect::<Vec<_>>();
                assert_eq!(mesh_objects.len(), 1);

                // Verify 2 color groups
                let color_groups: Vec<_> = get_color_groups_from_model(&package.root).collect();
                assert_eq!(color_groups.len(), 1);

                // Verify first color group (id=2) with 4 colors
                let first_group = color_groups
                    .iter()
                    .find(|cg| cg.colorgroup.id == 2)
                    .unwrap();
                assert_eq!(first_group.colorgroup.color.len(), 4);

                // Verify color values using Color type
                let expected_colors = [
                    Color::from_hex("#FF0000FF").unwrap(), // Red
                    Color::from_hex("#0000FFFF").unwrap(), // Blue
                    Color::from_hex("#00FF00FF").unwrap(), // Green
                    Color::from_hex("#FFFFFFFF").unwrap(), // White
                ];
                for (i, color_element) in first_group.colorgroup.color.iter().enumerate() {
                    assert_eq!(color_element.color, expected_colors[i]);
                }

                // Verify triangles with vertex colors point to valid color resources
                let mesh = &mesh_objects[0].mesh();
                let triangles_with_color: Vec<_> = mesh
                    .triangles
                    .triangle
                    .iter()
                    .filter(|t| {
                        t.pid.is_some() && (t.p1.is_some() || t.p2.is_some() || t.p3.is_some())
                    })
                    .collect();
                assert!(
                    !triangles_with_color.is_empty(),
                    "No triangles with vertex colors found"
                );

                // Verify each triangle with colors points to the color group
                for triangle in &triangles_with_color {
                    let pid = triangle.pid.get().unwrap();
                    // Check at least one of p1, p2, p3 points to a valid color index
                    let has_valid_index = [triangle.p1, triangle.p2, triangle.p3]
                        .iter()
                        .filter_map(|p| p.get())
                        .any(|pindex| {
                            let pindex_usize = pindex as usize;
                            pindex_usize < first_group.colorgroup.color.len()
                        });
                    assert_eq!(pid, 2, "Triangle pid should reference color group 2");
                    assert!(
                        has_valid_index,
                        "Triangle should have at least one valid color index"
                    );
                }

                // Verify material namespace is present
                let ns = package.get_namespaces_on_model(None).unwrap();
                assert!(ns.iter().any(|n| n.uri.contains("material")));
            }
            Err(err) => {
                panic!("read failed {:?}", err);
            }
        }
    }
}
