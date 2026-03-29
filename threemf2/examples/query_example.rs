//! Example demonstrating the query API for inspecting 3MF packages.
//!
//! This example shows common patterns for querying objects, items, and
//! relationships in 3MF packages, including multi-model scenarios.
//!
//! Run with:
//! ```bash
//! cargo run --example query_example --no-default-features --features io-write
//! ```

use threemf2::io::{ThreemfPackage, query::*};

use std::{fs::File, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 3MF Query API Example ===\n");

    // Load a test package with multiple models
    let path = PathBuf::from("tests/data/mesh-composedpart-beamlattice-separate-model-files.3mf");
    let file = File::open(&path)?;
    let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true)?;

    println!("Loaded package: {:?}\n", path.file_name().unwrap());

    // Section 1: Querying all objects
    section_1_querying_objects(&package);

    // Section 2: Querying build items
    section_2_querying_build_items(&package);

    // Section 3: Working with composed parts
    section_3_composed_parts(&package);

    // Section 4: Multi-model packages
    section_4_multi_model_packages(&package);

    // Section 5: Working with mesh objects
    section_5_mesh_objects(&package);

    // Section 6: Production extension features (if available)
    section_6_production_extension(&package);

    Ok(())
}

fn section_1_querying_objects(package: &ThreemfPackage) {
    println!("=== Section 1: Querying Objects ===");

    // Get all objects across all models
    let all_objects: Vec<_> = get_objects(package).collect();
    println!("Total objects in package: {}", all_objects.len());

    // Iterate and display object details
    for (i, obj_ref) in all_objects.iter().enumerate().take(5) {
        println!("\nObject {}:", i + 1);
        println!("  ID: {}", obj_ref.object.id);

        if let Some(name) = &obj_ref.object.name {
            println!("  Name: {}", name);
        }

        if let Some(path) = obj_ref.path {
            println!("  From sub-model: {}", path);
        } else {
            println!("  From root model");
        }

        // Check object type
        if obj_ref.object.kind.as_ref().unwrap().get_mesh().is_some() {
            println!("  Type: Mesh Object");
        } else if obj_ref
            .object
            .kind
            .as_ref()
            .unwrap()
            .get_components_object()
            .is_some()
        {
            println!("  Type: Composed Part");
        }
    }

    // Find specific object by ID
    if let Some(obj) = get_object_from_model(1, &package.root) {
        println!("\n✓ Found object with ID 1 in root model");
        if let Some(name) = &obj.object.name {
            println!("  Name: {}", name);
        }
    }

    println!();
}

fn section_2_querying_build_items(package: &ThreemfPackage) {
    println!("=== Section 2: Querying Build Items ===");

    // Get all build items
    let items: Vec<_> = get_items(package).collect();
    println!("Total build items: {}", items.len());

    // Display item details
    for (i, item_ref) in items.iter().enumerate() {
        println!("\nBuild Item {}:", i + 1);
        println!("  References object ID: {}", item_ref.objectid());

        if let Some(partnumber) = item_ref.partnumber() {
            println!("  Part number: {}", partnumber);
        }

        if let Some(transform) = item_ref.transform() {
            println!("  Has transform: {:?}", &transform.0[..3]); // Show first 3 values
        }

        if let Some(path) = item_ref.origin_model_path {
            println!("  From model: {}", path);
        }

        if let Some(uuid) = item_ref.uuid() {
            println!("  UUID: {}", uuid);
        }
    }

    // Find items that reference a specific object
    if !items.is_empty() {
        let first_objectid = items[0].objectid();
        let items_for_object: Vec<_> = get_items_by_objectid(package, first_objectid).collect();
        println!(
            "\n✓ Found {} item(s) referencing object {}",
            items_for_object.len(),
            first_objectid
        );
    }

    println!();
}

fn section_3_composed_parts(package: &ThreemfPackage) {
    println!("=== Section 3: Composed Parts & Components ===");

    let composed_parts: Vec<_> = get_components_objects(package).collect();
    println!("Total composed parts: {}", composed_parts.len());

    for (i, composed) in composed_parts.iter().enumerate() {
        println!("\nComposed Part {}:", i + 1);
        println!("  ID: {}", composed.id);

        if let Some(name) = &composed.name {
            println!("  Name: {}", name);
        }

        // Iterate components
        let components: Vec<_> = composed.components().collect();
        println!("  Components: {}", components.len());

        for (j, component) in components.iter().enumerate().take(3) {
            println!(
                "    Component {}: references object {}",
                j + 1,
                component.objectid
            );

            if let Some(path) = &component.path_to_look_for {
                println!("      Look in model: {}", path);
            }

            if component.transform.is_some() {
                println!("      Has transform");
            }

            if let Some(uuid) = &component.uuid {
                println!("      UUID: {}", uuid);
            }
        }

        if components.len() > 3 {
            println!("    ... and {} more components", components.len() - 3);
        }
    }

    println!();
}

fn section_4_multi_model_packages(package: &ThreemfPackage) {
    println!("=== Section 4: Multi-Model Packages ===");

    let models: Vec<_> = iter_models(package).collect();
    println!("Total models in package: {}", models.len());

    for (i, model_ref) in models.iter().enumerate() {
        if let Some(path) = model_ref.path {
            println!("\nSub-model {}: {}", i, path);
        } else {
            println!("\nRoot model:");
        }

        // Query objects in this specific model
        let objects_in_model: Vec<_> = get_objects_from_model(model_ref.model).collect();
        println!("  Objects: {}", objects_in_model.len());

        // Query items in this specific model
        let items_in_model: Vec<_> = get_items_from_model(model_ref.model).collect();
        println!("  Build items: {}", items_in_model.len());
    }

    // Count root vs sub-model entities
    let root_items = get_items(package)
        .filter(|i| i.origin_model_path.is_none())
        .count();
    let sub_model_items = get_items(package)
        .filter(|i| i.origin_model_path.is_some())
        .count();

    println!("\n✓ Items in root model: {}", root_items);
    println!("✓ Items in sub-models: {}", sub_model_items);

    println!();
}

fn section_5_mesh_objects(package: &ThreemfPackage) {
    println!("=== Section 5: Working with Mesh Objects ===");

    let mesh_objects: Vec<_> = get_mesh_objects(package).collect();
    println!("Total mesh objects: {}", mesh_objects.len());

    for (i, mesh_ref) in mesh_objects.iter().enumerate().take(3) {
        println!("\nMesh Object {}:", i + 1);
        println!("  ID: {}", mesh_ref.id);

        if let Some(name) = &mesh_ref.name {
            println!("  Name: {}", name);
        }

        // Access mesh data
        let mesh = mesh_ref.mesh();
        println!("  Vertices: {}", mesh.vertices.vertex.len());
        println!("  Triangles: {}", mesh.triangles.triangle.len());

        // Check for beam lattice extension
        if let Some(beamlattice) = &mesh.beamlattice {
            println!("  Has beam lattice:");
            println!("    Beams: {}", beamlattice.beams.beam.len());
            println!("    Min length: {}", beamlattice.minlength);
            println!("    Radius: {}", beamlattice.radius);
        }

        // Check for material properties
        if mesh_ref.pid.is_some() {
            println!("  Has material properties (pid: {:?})", mesh_ref.pid);
        }
    }

    if mesh_objects.len() > 3 {
        println!("\n... and {} more mesh objects", mesh_objects.len() - 3);
    }

    println!();
}

fn section_6_production_extension(package: &ThreemfPackage) {
    println!("=== Section 6: Production Extension (UUIDs) ===");

    // Check for UUIDs on build items
    let items_with_uuid: Vec<_> = get_items(package).filter(|i| i.uuid().is_some()).collect();

    println!("Items with UUIDs: {}", items_with_uuid.len());

    if let Some(item) = items_with_uuid.first() {
        println!("\nExample item with UUID:");
        println!("  Object ID: {}", item.objectid());
        println!("  UUID: {}", item.uuid().unwrap());

        // Try finding by UUID
        if let Some(found) = get_item_by_uuid(package, item.uuid().unwrap()) {
            println!("  ✓ Successfully found item by UUID lookup");
            println!("    References object: {}", found.objectid());
        }
    }

    // Check for UUIDs on objects
    let objects_with_uuid = get_objects(package)
        .filter(|o| o.object.uuid.is_some())
        .count();

    println!("\nObjects with UUIDs: {}", objects_with_uuid);

    // Check build UUID
    if let Some(build_uuid) = &package.root.build.uuid {
        println!("Root build UUID: {}", build_uuid);
    }

    println!();
}
