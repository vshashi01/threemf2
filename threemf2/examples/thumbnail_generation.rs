//! Example: Thumbnail Generation
//!
//! This example demonstrates how to generate a thumbnail from a 3MF model
//! and save it to a file.
//!
//! Run with: cargo run --example thumbnail_generation --features thumbnail-generation

use threemf2::{
    core::model::Model,
    io::{ThumbnailConfig, ThumbnailGenerator},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple cube model for demonstration
    let model = create_cube_model();

    // Configure the thumbnail generator
    let config = ThumbnailConfig::new()
        .with_dimensions(256, 256) // 256x256 pixels
        .with_padding(0.1) // 10% padding around the model
        .with_background_color(240, 240, 240, 255) // Light gray background
        .with_mesh_color(100, 149, 237, 255) // Cornflower blue mesh
        .with_camera_angles(45.0, 30.0); // Camera angles in degrees

    // Create the generator and generate the thumbnail
    let generator = ThumbnailGenerator::new(config);
    let thumbnail = generator.generate(&model)?;

    // Save the thumbnail to a file
    std::fs::write("thumbnail_output.png", &thumbnail.data)?;

    println!("Thumbnail generated successfully!");
    println!("Saved to: thumbnail_output.png");
    println!("Format: {:?}", thumbnail.format);
    println!("Size: {} bytes", thumbnail.data.len());

    // Demonstrate using the ThreemfPackage integration
    println!("\n--- Using ThreemfPackage integration ---");
    use threemf2::io::ThreemfPackage;

    let mut package: ThreemfPackage = model.into();
    package.generate_thumbnail(ThumbnailConfig::default(), "/Thumbnails/thumbnail.png")?;

    println!("Thumbnail added to ThreemfPackage");
    println!("Package now has {} thumbnail(s)", package.thumbnails.len());

    Ok(())
}

/// Creates a simple cube model for demonstration
fn create_cube_model() -> Model {
    use threemf2::core::build::{Build, Item};
    use threemf2::core::mesh::{Mesh, Triangle, Triangles, Vertex, Vertices};
    use threemf2::core::object::Object;
    use threemf2::core::resources::Resources;
    use threemf2::core::types::{OptionalResourceId, OptionalResourceIndex};

    // Define vertices for a cube
    let vertices = Vertices {
        vertex: vec![
            Vertex::new(-1.0, -1.0, -1.0), // 0
            Vertex::new(1.0, -1.0, -1.0),  // 1
            Vertex::new(1.0, 1.0, -1.0),   // 2
            Vertex::new(-1.0, 1.0, -1.0),  // 3
            Vertex::new(-1.0, -1.0, 1.0),  // 4
            Vertex::new(1.0, -1.0, 1.0),   // 5
            Vertex::new(1.0, 1.0, 1.0),    // 6
            Vertex::new(-1.0, 1.0, 1.0),   // 7
        ],
    };

    // Define triangles for the cube (6 faces * 2 triangles each = 12 triangles)
    let triangles = Triangles {
        triangle: vec![
            // Front face
            Triangle {
                v1: 0,
                v2: 1,
                v3: 2,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 0,
                v2: 2,
                v3: 3,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            // Back face
            Triangle {
                v1: 5,
                v2: 4,
                v3: 7,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 5,
                v2: 7,
                v3: 6,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            // Top face
            Triangle {
                v1: 3,
                v2: 2,
                v3: 6,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 3,
                v2: 6,
                v3: 7,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            // Bottom face
            Triangle {
                v1: 4,
                v2: 5,
                v3: 1,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 4,
                v2: 1,
                v3: 0,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            // Left face
            Triangle {
                v1: 4,
                v2: 0,
                v3: 3,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 4,
                v2: 3,
                v3: 7,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            // Right face
            Triangle {
                v1: 1,
                v2: 5,
                v3: 6,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
            Triangle {
                v1: 1,
                v2: 6,
                v3: 2,
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                p3: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            },
        ],
    };

    let mesh = Mesh {
        vertices,
        triangles,
        trianglesets: None,
        beamlattice: None,
    };

    let object = Object {
        id: 1,
        mesh: Some(mesh),
        components: None,
        name: Some("Cube".to_string()),
        pid: OptionalResourceId::none(),
        pindex: OptionalResourceIndex::none(),
        thumbnail: None,
        partnumber: None,
        uuid: None,
        objecttype: None,
    };

    let resources = Resources {
        object: vec![object],
        basematerials: vec![],
    };

    let build = Build {
        uuid: None,
        item: vec![Item {
            objectid: 1,
            transform: None,
            partnumber: None,
            uuid: None,
            path: None,
        }],
    };

    Model {
        unit: None,
        metadata: vec![],
        resources,
        build,
        recommendedextensions: None,
        requiredextensions: None,
    }
}
