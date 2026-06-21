use threemf2::model::{
    builder::{ModelBuilder, ObjectType, Unit},
    query::get_model_view,
};

/// This is an example showing the core 3MF Types available without any default features.
/// With no default features only the core structs are available from this library.
/// run with
/// `cargo run --example core-no-default-features --no-default-features`
///
fn main() {
    let mut builder = ModelBuilder::new(Unit::Inch, true);
    builder
        .add_metadata("Test metadata", None)
        .add_build(None)
        .unwrap();

    let obj_id = builder
        .add_mesh_object(|obj| {
            obj.object_type(ObjectType::Model);
            obj.add_vertices(&[[0.0, 0.0, 0.0], [-1.0, 0.0, 0.0], [-1.0, 1.0, 0.0]]);
            obj.add_triangle(&[0, 1, 2]);
            Ok(())
        })
        .unwrap();

    builder.add_build_item(obj_id).unwrap();
    let model = builder.build().unwrap();

    println!(
        "Number of build items: {}",
        get_model_view(&model).build_item_count()
    );
}
