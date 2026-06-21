use std::{fs::File, path::PathBuf};
use threemf2::{
    io::{CachePolicy, Error, ThreemfPackageLazyReader},
    model::PathResource,
};

/// This example demonstrates extracting raw XML strings from a 3MF package
/// using the pull-based reader with string extraction methods.
///
/// Run with:
/// `cargo run --example string_extraction --features io-lazy-read`
fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/data/P_XPX_0702_02.3mf");
    let reader = File::open(path).unwrap();

    let package = ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
        reader,
        CachePolicy::NoCache,
    )
    .unwrap();

    println!("=== 3MF Package String Extraction Example ===\n");

    println!("1. Root Model XML (first 200 chars):");
    package
        .with_model_xml(package.root_model_path(), |xml| {
            println!("{}...\n", &xml[..xml.len().min(200)]);
        })
        .unwrap();

    println!("2. Root Relationships XML:");
    package
        .with_relationships_xml(&PathResource::new("_rels/.rels", true).unwrap(), |xml| {
            println!("{}\n", xml);
        })
        .unwrap();

    println!("3. Content Types XML:");
    package
        .with_content_types_xml(|xml| {
            println!("{}\n", xml);
        })
        .unwrap();

    println!("4. Sub-model XML (first 200 chars):");
    package
        .with_model_xml(
            &PathResource::new("/3D/midway.model", true).unwrap(),
            |xml| {
                println!("{}...\n", &xml[..xml.len().min(200)]);
            },
        )
        .unwrap();

    // Show that invalid paths return errors
    println!("5. Invalid path handling:");
    match package.with_model_xml(
        &PathResource::new("/invalid/path.model", true).unwrap(),
        |_| "found",
    ) {
        Ok(result) => println!("Unexpected success: {}", result),
        Err(Error::ResourceNotFound(msg)) => println!("Model not found: {}\n", msg),
        Err(other) => println!("Other error: {:?}\n", other),
    }

    match package.with_relationships_xml(
        &PathResource::new("/invalid/rels.xml", true).unwrap(),
        |_| "found",
    ) {
        Ok(result) => println!("Unexpected success: {}", result),
        Err(Error::ResourceNotFound(msg)) => println!("Relationships not found: {}\n", msg),
        Err(other) => println!("Other error: {:?}\n", other),
    }
}
