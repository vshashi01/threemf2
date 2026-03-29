#[cfg(test)]
mod tests {
    use instant_xml::from_str;
    use std::path::PathBuf;
    use threemf2::core::model::Model;
    use threemf2::io::query::*;

    #[test]
    fn test_get_object_ref_from_model() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/lfs/mesh-composedpart-beamlattice.model");
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let object_ref = get_object_from_model(1, &model);

        match object_ref {
            Some(obj_ref) => {
                assert!(obj_ref.object.kind.as_ref().unwrap().get_mesh().is_some());
                assert_eq!(obj_ref.object.id, 1);
            }
            None => panic!("Object ref not found"),
        }
    }

    #[test]
    fn test_get_objects_from_model() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/lfs/mesh-composedpart-beamlattice.model");
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let objects = get_objects_from_model(&model).collect::<Vec<_>>();
        assert_eq!(objects.len(), 6);
    }

    #[test]
    fn test_get_mesh_objects_from_model() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/lfs/mesh-composedpart-beamlattice.model");
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let objects = get_mesh_objects_from_model(&model).collect::<Vec<_>>();
        assert_eq!(objects.len(), 5);
    }

    #[test]
    fn test_get_composedpart_objects_from_model() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/lfs/mesh-composedpart-beamlattice.model");
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let objects = get_components_objects_from_model(&model).collect::<Vec<_>>();
        assert_eq!(objects.len(), 1)
    }

    #[test]
    fn test_get_beamlattice_objects_from_model() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/lfs/mesh-composedpart-beamlattice.model");
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let objects = get_mesh_objects_from_model(&model)
            .filter(|mesh_ref| mesh_ref.mesh().beamlattice.is_some())
            .count();
        assert_eq!(objects, 2)
    }

    #[test]
    fn test_get_object_from_model_non_existent_id() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/lfs/mesh-composedpart-beamlattice.model");
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let object_ref = get_object_from_model(999, &model);
        assert!(object_ref.is_none());
    }

    #[test]
    fn test_get_objects_from_model_ref() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/lfs/mesh-composedpart-beamlattice.model");
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();
        let model_ref = ModelRef {
            model: &model,
            path: Some("test_path"),
        };

        let objects = get_objects_from_model_ref(model_ref).collect::<Vec<_>>();
        assert_eq!(objects.len(), 6);
        for obj in objects {
            assert_eq!(obj.path, Some("test_path"));
        }
    }

    #[test]
    fn test_get_mesh_objects_from_model_ref() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/lfs/mesh-composedpart-beamlattice.model");
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();
        let model_ref = ModelRef {
            model: &model,
            path: None,
        };

        let objects = get_mesh_objects_from_model_ref(model_ref).collect::<Vec<_>>();
        assert_eq!(objects.len(), 5);
        for obj in objects {
            assert!(obj.object.kind.as_ref().unwrap().get_mesh().is_some());
        }
    }

    #[test]
    fn test_mesh_object_ref_impl() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/lfs/mesh-composedpart-beamlattice.model");
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let mesh_objects = get_mesh_objects_from_model(&model).collect::<Vec<_>>();
        assert!(!mesh_objects.is_empty());
        let mesh_ref = &mesh_objects[0];
        assert_eq!(mesh_ref.id, 1);
        assert!(!mesh_ref.mesh().vertices.vertex.is_empty());
    }

    #[test]
    fn test_composedpart_object_ref_impl() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/lfs/mesh-composedpart-beamlattice.model");
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let composedpart_objects = get_components_objects_from_model(&model).collect::<Vec<_>>();
        assert!(!composedpart_objects.is_empty());
        let composed_part = &composedpart_objects[0];
        assert_eq!(composed_part.id, 4);
        assert_eq!(composed_part.components().count(), 2);

        for comp in composed_part.components() {
            assert!(comp.objectid > 0);
        }
    }

    #[test]
    fn test_beam_lattice_object_ref_impl() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/lfs/mesh-composedpart-beamlattice.model");
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let beam_objects = get_mesh_objects_from_model(&model)
            .filter(|mesh_ref| mesh_ref.mesh().beamlattice.is_some())
            .collect::<Vec<_>>();
        assert!(!beam_objects.is_empty());
        let mesh_ref = &beam_objects[0];
        assert_eq!(mesh_ref.id, 5);
        assert!(
            !mesh_ref
                .mesh()
                .beamlattice
                .as_ref()
                .unwrap()
                .beams
                .beam
                .is_empty()
        );
    }

    #[test]
    fn test_model_ref_fields() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/lfs/mesh-composedpart-beamlattice.model");
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let model_ref = ModelRef {
            model: &model,
            path: Some("sub/model.model"),
        };
        assert_eq!(model_ref.path, Some("sub/model.model"));
        assert_eq!(model_ref.model as *const _, &model as *const _);
    }

    #[test]
    fn test_get_items_from_model() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/lfs/mesh-composedpart-beamlattice.model");
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let items = get_items_from_model(&model).collect::<Vec<_>>();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].objectid(), 1);
        assert!(items[0].origin_model_path.is_none());
    }

    #[test]
    fn test_get_items_from_model_ref() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/lfs/mesh-composedpart-beamlattice.model");
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();
        let model_ref = ModelRef {
            model: &model,
            path: Some("sub/model.model"),
        };

        let items = get_items_from_model_ref(model_ref).collect::<Vec<_>>();
        assert_eq!(items.len(), 4);
        assert_eq!(items[0].origin_model_path, Some("sub/model.model"));
    }

    #[test]
    fn test_item_ref_methods() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/data/lfs/mesh-composedpart-beamlattice.model");
        let text = std::fs::read_to_string(path).unwrap();
        let model = from_str::<Model>(&text).unwrap();

        let items = get_items_from_model(&model).collect::<Vec<_>>();
        assert_eq!(items.len(), 4);
        let item_ref = &items[0];
        assert_eq!(item_ref.objectid(), 1);
        assert!(item_ref.transform().is_some());
        assert_eq!(item_ref.partnumber(), Some("Pyramid"));
        assert!(item_ref.path().is_none());
        assert_eq!(
            item_ref.uuid(),
            Some("4e44739e-3ba0-4639-b8ad-1eb80b1cb5a5")
        );
    }
}
