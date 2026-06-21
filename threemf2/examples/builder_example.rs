use threemf2::io::ThreemfPackageBuilder;
use threemf2::model::{
    builder::{ModelBuilder, ObjectType, Unit},
    query::{get_mesh_objects_from_model, get_model_view, get_objects_from_model},
};

/// This example shows how to build 3MF Model using ModelBuilder.
/// Use this to reduce the boilerplate needed to setup 3MF Models.
///
/// Run with:
/// `cargo run --example builder_example --no-default-features --features io-write`
///
fn main() {
    // Create a simple cube model using the builder
    let mut builder = ModelBuilder::new(Unit::Millimeter, true);

    // Set model properties
    builder
        .unit(Unit::Millimeter)
        .add_metadata("Application", Some("Builder Example"))
        .add_metadata(
            "Description",
            Some("A simple cube created with ModelBuilder"),
        );

    // Add a cube object
    let cube_id = builder.add_mesh_object(|obj| {
        let _ = obj
            .name("Cube")
            .object_type(ObjectType::Model)
            .part_number("CUBE-001");
        //.mesh(|mesh| {
        // Define vertices for a cube
        obj.add_vertex(&[0.0, 0.0, 0.0]) // 0: bottom-front-left
            .add_vertex(&[10.0, 0.0, 0.0]) // 1: bottom-front-right
            .add_vertex(&[10.0, 10.0, 0.0]) // 2: bottom-back-right
            .add_vertex(&[0.0, 10.0, 0.0]) // 3: bottom-back-left
            .add_vertex(&[0.0, 0.0, 10.0]) // 4: top-front-left
            .add_vertex(&[10.0, 0.0, 10.0]) // 5: top-front-right
            .add_vertex(&[10.0, 10.0, 10.0]) // 6: top-back-right
            .add_vertex(&[0.0, 10.0, 10.0]); // 7: top-back-left

        // Define triangles for the cube faces
        // Bottom face
        obj.add_triangle(&[0, 1, 2]).add_triangle(&[0, 2, 3]);
        // Top face
        obj.add_triangle(&[4, 5, 6]).add_triangle(&[4, 6, 7]);
        // Front face
        obj.add_triangle(&[0, 1, 5]).add_triangle(&[0, 5, 4]);
        // Back face
        obj.add_triangle(&[3, 2, 6]).add_triangle(&[3, 6, 7]);
        // Left face
        obj.add_triangle(&[0, 3, 7]).add_triangle(&[0, 7, 4]);
        // Right face
        obj.add_triangle(&[1, 2, 6]).add_triangle(&[1, 6, 5]);
        //});

        Ok(())
    });

    // Add the Build to the model
    // Note: A root model always needs a build item.
    if let Err(err) = builder.add_build(None) {
        panic!("{err:?}");
    }

    match cube_id {
        Ok(id) => {
            // Add the cube to the build
            if let Err(err) = builder.add_build_item(id) {
                panic!("{err:?}");
            }
        }
        Err(err) => panic!("{err:?}"),
    }

    // Build the final model
    let model = builder.build();

    match model {
        Ok(model) => {
            println!("Model created successfully!");
            let model_view = get_model_view(&model);
            println!("Unit: {:?}", model_view.unit());
            println!("Metadata count: {}", model_view.metadata_count());
            println!("Objects count: {}", model_view.object_count());
            println!("Build items count: {}", model_view.build_item_count());

            if let Some(obj) = get_objects_from_model(&model).next() {
                println!("First object name: {:?}", obj.name());
            }

            if let Some(mesh) = get_mesh_objects_from_model(&model).next() {
                println!("Vertices: {}", mesh.vertex_count());
                println!("Triangles: {}", mesh.triangle_count());
            }

            //to create a 3MF Package easily use the package builder
            let mut package_builder = ThreemfPackageBuilder::new();
            package_builder.set_root_model(model);
            let _package = package_builder.build().expect("Error building package");
        }
        Err(err) => panic!("{err:?}"),
    }
}
