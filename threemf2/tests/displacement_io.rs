#[cfg(any(
    feature = "io-memory-optimized-read",
    feature = "io-speed-optimized-read",
    feature = "io-lazy-read"
))]
#[cfg(test)]
mod tests {

    use threemf2::model::query as core_query;
    use threemf2::package::ThreemfPackage;
    use threemf2::package::query as io_query;

    use std::{fs::File, path::PathBuf};

    #[cfg(feature = "io-memory-optimized-read")]
    #[test]
    fn read_displacement_package_memory_optimized() {
        use threemf2::threemf_namespaces::ThreemfNamespace;

        let path =
            PathBuf::from("./tests/data/mgx-core-prod-beamlattice-material-displacement-mesh.3mf");
        let reader = File::open(path).unwrap();

        let package =
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, true).unwrap();

        assert_eq!(io_query::get_displacement_mesh_objects(&package).count(), 1);
        for object in io_query::get_displacement_mesh_objects(&package) {
            assert!(object.view.has_beamlattice())
        }
        assert!(core_query::get_displacement2d_by_id(3, &package.root).is_some());
        assert!(core_query::get_normvectorgroup_by_id(4, &package.root).is_some());
        assert!(core_query::get_disp2dgroup_by_id(5, &package.root).is_some());

        let namespaces = package.get_namespaces_on_model(None).unwrap();
        assert!(
            namespaces
                .iter()
                .any(|ns| matches!(ns, ThreemfNamespace::Displacement))
        );
    }

    #[cfg(feature = "io-speed-optimized-read")]
    #[test]
    fn read_displacement_package_speed_optimized() {
        use threemf2::threemf_namespaces::ThreemfNamespace;

        let path =
            PathBuf::from("./tests/data/mgx-core-prod-beamlattice-material-displacement-mesh.3mf");
        let reader = File::open(path).unwrap();

        let package =
            ThreemfPackage::from_reader_with_speed_optimized_deserializer(reader, true).unwrap();

        assert_eq!(io_query::get_displacement_mesh_objects(&package).count(), 1);
        for object in io_query::get_displacement_mesh_objects(&package) {
            assert!(object.view.has_beamlattice())
        }
        assert!(core_query::get_displacement2d_by_id(3, &package.root).is_some());
        assert!(core_query::get_normvectorgroup_by_id(4, &package.root).is_some());
        assert!(core_query::get_disp2dgroup_by_id(5, &package.root).is_some());

        let namespaces = package.get_namespaces_on_model(None).unwrap();
        assert!(
            namespaces
                .iter()
                .any(|ns| matches!(ns, ThreemfNamespace::Displacement))
        );
    }

    #[cfg(all(feature = "io-lazy-read", feature = "io-memory-optimized-read"))]
    #[test]
    fn read_displacement_package_lazy_memory_optimized() {
        use threemf2::{
            model::PathResource,
            package::{CachePolicy, ThreemfPackageLazyReader},
        };

        let path =
            PathBuf::from("./tests/data/mgx-core-prod-beamlattice-material-displacement-mesh.3mf");
        let reader = File::open(path).unwrap();

        let package = ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
            reader,
            CachePolicy::NoCache,
        )
        .unwrap();

        package
            .with_model(
                &PathResource::new("/3D/3dmodel.model", true).unwrap(),
                |model| {
                    assert_eq!(
                        core_query::get_displacement_mesh_objects_from_model(model).count(),
                        1
                    );
                    assert!(core_query::get_displacement2d_by_id(3, model).is_some());
                    assert!(core_query::get_normvectorgroup_by_id(4, model).is_some());
                    assert!(core_query::get_disp2dgroup_by_id(5, model).is_some());
                },
            )
            .unwrap();
    }
}
