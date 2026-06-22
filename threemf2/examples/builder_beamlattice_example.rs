use threemf2::model::{
    builder::{BallMode, CapMode, ModelBuilder, ObjectType, Unit},
    query::{get_mesh_objects_from_model, get_model_view},
};

/// This example demonstrates how to use the BeamLatticeBuilder with ModelBuilder
/// to create 3MF models with beam lattice structures.
///
/// Run with:
/// `cargo run --example builder_beamlattice_example --no-default-features --features package-write`
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
            let model_view = get_model_view(&model);
            println!("  Unit: {:?}", model_view.unit());
            println!("  Metadata count: {}", model_view.metadata_count());
            println!("  Objects count: {}", model_view.object_count());
            println!("  Build items count: {}", model_view.build_item_count());
            println!();

            if let Some(mesh) = get_mesh_objects_from_model(&model).next() {
                println!("Vertices: {}", mesh.vertex_count());
                println!("Triangles: {}", mesh.triangle_count());

                if let Some(lattice) = mesh.lattice() {
                    let data = lattice.data();
                    println!();
                    println!("Beam Lattice Properties:");
                    println!("  Min length: {}", data.minlength);
                    println!("  Default radius: {}", data.radius);
                    println!("  Ball mode: {:?}", data.ball_mode);
                    println!("  Ball radius: {:?}", data.ball_radius);
                    println!("  Clipping mode: {:?}", data.clippingmode);
                    println!("  Beams: {}", data.beam_count);
                    println!("  Balls: {}", data.ball_count);

                    if let Some(beamsets) = lattice.beamsets() {
                        let beamset_count = lattice.beamset_count();
                        println!("  Beam sets: {}", beamset_count);
                        for (i, bs) in beamsets.enumerate() {
                            println!(
                                "    Set {}: {:?} ({} beams, {} balls)",
                                i,
                                bs.name(),
                                bs.beam_refs().count(),
                                bs.ball_refs().count()
                            );
                        }
                    }
                }
            }
        }
        Err(err) => panic!("Failed to build model: {err:?}"),
    }
}
