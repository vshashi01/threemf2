use threemf2::io::{BallMode, CapMode, ModelBuilder, ObjectType, Unit};

/// This example demonstrates how to use the BeamLatticeBuilder with ModelBuilder
/// to create 3MF models with beam lattice structures.
///
/// Run with:
/// `cargo run --example builder_beamlattice_example --no-default-features --features io-write`
///
fn main() {
    // Create a model with beam lattice extension enabled
    let mut builder = ModelBuilder::new(Unit::Millimeter, true);

    // Set model properties
    builder
        .unit(Unit::Millimeter)
        .add_metadata("Application", Some("Beam Lattice Builder Example"))
        .add_metadata(
            "Description",
            Some("A simple lattice structure created with BeamLatticeBuilder"),
        );

    // Add a lattice object with vertices and beams
    let lattice_id = builder.add_mesh_object(|obj| {
        obj.name("Lattice Structure")
            .object_type(ObjectType::Model)
            .part_number("LATTICE-001");

        // Define vertices for a simple cubic lattice
        obj.add_vertices(&[
            [0.0, 0.0, 0.0],    // 0: bottom-front-left
            [10.0, 0.0, 0.0],   // 1: bottom-front-right
            [10.0, 10.0, 0.0],  // 2: bottom-back-right
            [0.0, 10.0, 0.0],   // 3: bottom-back-left
            [0.0, 0.0, 10.0],   // 4: top-front-left
            [10.0, 0.0, 10.0],  // 5: top-front-right
            [10.0, 10.0, 10.0], // 6: top-back-right
            [0.0, 10.0, 10.0],  // 7: top-back-left
        ]);

        // Add beam lattice to the mesh
        obj.add_beam_lattice(|bl| {
            // Configure beam lattice properties
            bl.minlength(0.001)
                .radius(1.0)
                .cap(CapMode::Sphere)
                .ballmode(BallMode::Mixed)
                .ballradius(0.5);

            // Add simple beams (edges of the cube)
            // Bottom face edges
            bl.add_beam(0, 1)
                .add_beam(1, 2)
                .add_beam(2, 3)
                .add_beam(3, 0);

            // Top face edges
            bl.add_beams(&[(4, 5), (5, 6), (6, 7), (7, 4)]);

            // Vertical edges
            bl.add_beam(0, 4)
                .add_beam(1, 5)
                .add_beam(2, 6)
                .add_beam(3, 7);

            // Add a beam with custom radii using advanced builder
            bl.add_beam_advanced(0, 6, |b| {
                b.radius_1(1.5).radius_2(2.0).cap_1(CapMode::Hemisphere)
            });

            // Add balls at specific vertices (mixed mode)
            bl.add_ball(0) // Corner ball with default radius
                .add_ball_advanced(6, |b| b.radius(0.75)); // Corner ball with custom radius

            // Add more balls in bulk
            bl.add_balls(&[1, 3, 5, 7]);

            // Create a beam set to group structural beams
            bl.add_beamset(|bs| {
                bs.name("Bottom Face")
                    .identifier("bottom-face-001")
                    .add_beam_refs(&[0, 1, 2, 3]) // Bottom face beam indices
                    .add_ball_refs(&[0, 1]); // Bottom corner balls
            });

            // Create another beam set for top face
            bl.add_beamset(|bs| {
                bs.name("Top Face")
                    .identifier("top-face-001")
                    .add_beam_refs(&[4, 5, 6, 7]); // Top face beam indices
            });
        });

        Ok(())
    });

    // Add build section
    if let Err(err) = builder.add_build(None) {
        panic!("Failed to add build: {err:?}");
    }

    // Add lattice to build
    match lattice_id {
        Ok(id) => {
            if let Err(err) = builder.add_build_item(id) {
                panic!("Failed to add build item: {err:?}");
            }
        }
        Err(err) => panic!("Failed to create lattice object: {err:?}"),
    }

    // Build the final model
    let model = builder.build();

    match model {
        Ok(model) => {
            println!("✓ Beam lattice model created successfully!");
            println!();
            println!("Model Properties:");
            println!("  Unit: {:?}", model.unit);
            println!("  Metadata count: {}", model.metadata.len());
            println!("  Objects count: {}", model.resources.object.len());
            println!("  Build items count: {}", model.build.item.len());
            println!();

            if let Some(obj) = model.resources.object.first() {
                println!("Object: {:?}", obj.name);
                if let Some(mesh) = &obj.get_mesh() {
                    println!("  Vertices: {}", mesh.vertices.vertex.len());
                    println!("  Triangles: {}", mesh.triangles.triangle.len());

                    if let Some(bl) = &mesh.beamlattice {
                        println!();
                        println!("Beam Lattice Properties:");
                        println!("  Min length: {}", bl.minlength);
                        println!("  Default radius: {}", bl.radius);
                        println!("  Ball mode: {:?}", bl.ballmode);
                        println!("  Ball radius: {:?}", bl.ballradius);
                        println!("  Cap mode: {:?}", bl.cap);
                        println!("  Beams: {}", bl.beams.beam.len());

                        if let Some(balls) = &bl.balls {
                            println!("  Balls: {}", balls.ball.len());
                        }

                        if let Some(beamsets) = &bl.beamsets {
                            println!("  Beam sets: {}", beamsets.beamset.len());
                            for (i, bs) in beamsets.beamset.iter().enumerate() {
                                println!(
                                    "    Set {}: {:?} ({} beams, {} balls)",
                                    i,
                                    bs.name,
                                    bs.refs.len(),
                                    bs.ballref.len()
                                );
                            }
                        }
                    }
                }
            }
        }
        Err(err) => panic!("Failed to build model: {err:?}"),
    }
}
