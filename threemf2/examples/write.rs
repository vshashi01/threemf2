use threemf2::model::builder::{ModelBuilder, ObjectType, Unit};
use threemf2::package::ThreemfPackageBuilder;

use std::{io::Cursor, vec};

/// This is an example showing the core 3MF Types available without any default features.
/// With no default features only the core structs are available from this library.
/// run with
/// `cargo run --example write --no-default-features --features package-write`
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
    let mut package_builder = ThreemfPackageBuilder::new();
    package_builder.set_root_model(model);
    let package = package_builder.build().expect("Error building package");

    let mut bytes: Vec<u8> = vec![];
    let writer = Cursor::new(&mut bytes);

    let result = package.write(writer);

    match result {
        Ok(_) => println!("The length of the ThreemfPackage in bytes: {}", bytes.len()),
        Err(err) => println!("Error writing the model into a ThreemfPackage: {:?}", err),
    }
}
