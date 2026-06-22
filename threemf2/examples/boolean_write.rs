//! Example: Creating a 3MF file with Boolean Operations
//!
//! This example demonstrates how to use the Boolean Operations extension
//! to create complex shapes through Constructive Solid Geometry (CSG).
//!
//! The example creates:
//! - A cube mesh (base object)
//! - A smaller cube mesh (operand)
//! - A boolean shape that subtracts the smaller cube from the base cube
//!
//! # Running the example
//!
//! ```bash
//! cargo run --example boolean_write --features "package-write"
//! ```

use std::io::Cursor;

use threemf2::{
    model::{
        builder::{BooleanOperation, ModelBuilder, ObjectType, Unit},
        query::get_model_view,
    },
    package::ThreemfPackageBuilder,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating 3MF file with boolean operations...");

    let mut builder = ModelBuilder::new(Unit::Millimeter, true);
    builder
        .add_metadata("Application", Some("Boolean Example"))
        .add_metadata("Description", Some("Cube 1 minus Cube 2 using CSG"))
        .add_build(None)?;

    let cube_triangles: [[usize; 3]; 12] = [
        [0, 1, 2],
        [0, 2, 3],
        [4, 6, 5],
        [4, 7, 6],
        [0, 5, 1],
        [0, 4, 5],
        [2, 7, 3],
        [2, 6, 7],
        [0, 7, 4],
        [0, 3, 7],
        [1, 6, 2],
        [1, 5, 6],
    ];

    let cube_id = builder.add_mesh_object(|obj| {
        obj.name("Cube")
            .object_type(ObjectType::Model)
            .add_vertices(&[
                [-5.0, -5.0, 0.0],
                [5.0, -5.0, 0.0],
                [5.0, 5.0, 0.0],
                [-5.0, 5.0, 0.0],
                [-5.0, -5.0, 10.0],
                [5.0, -5.0, 10.0],
                [5.0, 5.0, 10.0],
                [-5.0, 5.0, 10.0],
            ]);

        obj.add_triangles(&cube_triangles);
        Ok(())
    })?;

    let inner_id = builder.add_mesh_object(|obj| {
        obj.name("InnerCube")
            .object_type(ObjectType::Model)
            .add_vertices(&[
                [-2.0, -2.0, 3.0],
                [2.0, -2.0, 3.0],
                [2.0, 2.0, 3.0],
                [-2.0, 2.0, 3.0],
                [-2.0, -2.0, 7.0],
                [2.0, -2.0, 7.0],
                [2.0, 2.0, 7.0],
                [-2.0, 2.0, 7.0],
            ]);

        obj.add_triangles(&cube_triangles);
        Ok(())
    })?;

    let result_id = builder.add_booleanshape_object(|obj| {
        obj.name("CubeMinusCube");
        obj.base_object(cube_id, BooleanOperation::Difference);
        obj.add_boolean(inner_id);
        Ok(())
    })?;

    builder.add_build_item(result_id)?;

    println!("  Created cube mesh (object id: {})", cube_id.0);
    println!("  Created inner cube mesh (object id: {})", inner_id.0);
    println!("  Created boolean shape (object id: {})", result_id.0);
    println!("  Added boolean result to build plate");

    let model = builder.build()?;

    println!("\nModel statistics:");
    let model_view = get_model_view(&model);
    println!("  Objects: {}", model_view.object_count());
    println!("  Build items: {}", model_view.build_item_count());
    println!(
        "  Required extensions: {:?}",
        model_view.required_extensions()
    );

    let mut package_builder = ThreemfPackageBuilder::new();
    package_builder.set_root_model(model);
    let package = package_builder.build().expect("Error building package");
    let mut buffer = Cursor::new(Vec::new());
    package.write(&mut buffer)?;

    println!(
        "\nSuccessfully wrote 3MF data to buffer: {} bytes",
        buffer.get_ref().len()
    );
    println!("\nNote: This example uses simple cubes to approximate geometry.");

    Ok(())
}
