use threemf2::model::builder::{BallMode, CapMode, ModelBuilder, ObjectType, Unit};
use threemf2::package::ThreemfPackageBuilder;

use std::{io::Cursor, vec};

/// This example shows how to create and write a 3MF file with beam lattice structures.
/// This demonstrates the Beam Lattice Extension support in threemf2.
///
/// Run with:
/// `cargo run --example beamlattice_write --no-default-features --features package-write`
///
fn main() {
    let mut builder = ModelBuilder::new(Unit::Millimeter, true);
    builder
        .add_metadata("Beam Lattice Example", None)
        .add_build(None)
        .unwrap();

    let beam_defs = vec![
        (0, 1, Some(1.5), Some(1.6)),
        (2, 0, Some(3.0), Some(1.5)),
        (1, 3, Some(1.6), Some(3.0)),
        (3, 2, Some(3.0), None),
        (2, 4, Some(3.0), Some(2.0)),
        (4, 5, Some(2.0), None),
        (5, 6, Some(2.0), None),
        (7, 6, Some(2.0), None),
        (1, 6, Some(1.6), Some(2.0)),
        (7, 4, Some(2.0), None),
        (7, 3, Some(2.0), Some(3.0)),
        (0, 5, Some(1.5), Some(2.0)),
    ];

    let ball_defs = vec![(0, Some(0.5)), (5, None), (7, Some(0.5))];

    let object_id = builder
        .add_mesh_object(|obj| {
            obj.object_type(ObjectType::Model)
                .part_number("beam-lattice-example")
                .name("Beam Lattice Cube");

            obj.add_vertices(&[
                [45.0, 55.0, 55.0],
                [45.0, 45.0, 55.0],
                [45.0, 55.0, 45.0],
                [45.0, 45.0, 45.0],
                [55.0, 55.0, 45.0],
                [55.0, 55.0, 55.0],
                [55.0, 45.0, 55.0],
                [55.0, 45.0, 45.0],
            ]);

            obj.add_beam_lattice(|bl| {
                bl.minlength(0.0001)
                    .radius(1.0)
                    .ballmode(BallMode::Mixed)
                    .ballradius(0.25)
                    .cap(CapMode::Sphere);

                for (v1, v2, r1, r2) in &beam_defs {
                    bl.add_beam_advanced(*v1, *v2, |b| {
                        let b = if let Some(radius) = r1 {
                            b.radius_1(*radius)
                        } else {
                            b
                        };
                        if let Some(radius) = r2 {
                            b.radius_2(*radius)
                        } else {
                            b
                        }
                    });
                }

                for (vindex, radius) in &ball_defs {
                    bl.add_ball_advanced(*vindex, |b| {
                        if let Some(radius) = radius {
                            b.radius(*radius)
                        } else {
                            b
                        }
                    });
                }
            });

            Ok(())
        })
        .unwrap();

    builder.add_build_item(object_id).unwrap();
    let model = builder.build().unwrap();

    let mut package_builder = ThreemfPackageBuilder::new();
    package_builder.set_root_model(model);
    let package = package_builder.build().expect("Error building package");

    let mut bytes: Vec<u8> = vec![];
    let writer = Cursor::new(&mut bytes);

    let result = package.write(writer);

    match result {
        Ok(_) => {
            println!("Successfully created 3MF file with beam lattice!");
            println!("File size: {} bytes", bytes.len());
            println!("The file contains:");
            println!("  - {} beams with variable radii", 12);
            println!("  - {} balls at select vertices", 3);
            println!("  - Mixed ball mode with sphere capping");
            println!("  - Minimum beam length: 0.0001 mm");
            println!("  - Default beam radius: 1.0 mm");
            println!("  - Default ball radius: 0.25 mm");
        }
        Err(err) => println!("Error writing beam lattice 3MF file: {:?}", err),
    }
}
