use threemf2::{
    core::{
        builder::{ModelBuilder, ObjectType, Unit},
        slice::MeshResolution,
    },
    io::{
        ThreemfPackage,
        content_types::{ContentTypes, DefaultContentTypeEnum, DefaultContentTypes},
        relationship::{Relationship, RelationshipType, Relationships},
    },
};

use std::collections::HashMap;
use std::{io::Cursor, vec};

/// This example shows how to create and write a 3MF file with slice extension data.
/// This demonstrates the Slice Extension support in threemf2.
///
/// Run with:
/// `cargo run --example slice_write --no-default-features --features io-write`
///
fn main() {
    let mut builder = ModelBuilder::new(Unit::Millimeter, true);
    builder.add_build(None).unwrap();

    let slicestack_id = builder
        .add_slice_stack(|stack| {
            stack.zbottom(0.0);

            stack.add_slice(|slice| {
                slice.ztop(0.1);
                slice.add_vertices(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]);
                slice.add_polygon(|poly| {
                    poly.start_vertex(0);
                    poly.add_segment(1);
                    poly.add_segment(2);
                    poly.add_segment(3);
                });
            });

            stack.add_slice(|slice| {
                slice.ztop(0.2);
                slice.add_vertices(&[(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)]);
                slice.add_polygon(|poly| {
                    poly.start_vertex(0);
                    poly.add_segment(1);
                    poly.add_segment(2);
                    poly.add_segment(3);
                });
            });
        })
        .unwrap();

    let object_id = builder
        .add_mesh_object(|obj| {
            obj.object_type(ObjectType::Model)
                .part_number("SLICE_EXAMPLE_001")
                .name("SlicedCube")
                .slice_stack(slicestack_id, None, Some(MeshResolution::LowRes));

            obj.add_vertices(&[
                [0.0, 0.0, 0.0],
                [10.0, 0.0, 0.0],
                [10.0, 10.0, 0.0],
                [0.0, 10.0, 0.0],
                [0.0, 0.0, 10.0],
                [10.0, 0.0, 10.0],
                [10.0, 10.0, 10.0],
                [0.0, 10.0, 10.0],
            ]);

            obj.add_triangles(&[[0, 2, 1], [0, 3, 2], [4, 5, 6], [4, 6, 7]]);

            Ok(())
        })
        .unwrap();

    builder.add_build_item(object_id).unwrap();
    let model = builder.build().unwrap();

    let content_types = ContentTypes {
        defaults: vec![
            DefaultContentTypes {
                extension: "rels".to_owned(),
                content_type: DefaultContentTypeEnum::Relationship,
            },
            DefaultContentTypes {
                extension: "model".to_owned(),
                content_type: DefaultContentTypeEnum::Model,
            },
        ],
    };

    let relationships = HashMap::from([(
        "_rels/.rels".to_owned(),
        Relationships {
            relationships: vec![Relationship {
                id: "rel0".to_owned(),
                target: "3D/3Dmodel.model".to_owned(),
                relationship_type: RelationshipType::Model,
            }],
        },
    )]);

    let package = ThreemfPackage::new(
        model,
        HashMap::new(),
        HashMap::new(),
        HashMap::new(),
        relationships,
        content_types,
    );

    let mut buf = Cursor::new(Vec::new());
    package.write(&mut buf).expect("Error writing 3MF package");

    println!("Successfully created 3MF file with slice extension!");
    println!("Package size: {} bytes", buf.get_ref().len());
    println!();
    println!("This example demonstrates:");
    println!("  - Creating a SliceStack with multiple 2D slices");
    println!("  - Defining vertices and polygons for each slice layer");
    println!("  - Referencing slice data from an object");
    println!("  - Setting meshresolution to 'lowres' (requires slice extension)");
    println!("  - Automatic namespace and requiredextensions generation");
}
