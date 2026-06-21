use threemf2::{io::ThreemfPackage, model::query::get_model_view};

use std::{fs::File, path::PathBuf};

/// This is an example showing speed optimized reading (deprecated)
/// Run with:
/// `cargo run --example io_speed_optimized_read --features io-speed-optimized-read`
///
fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/mgx-core-prod-beamlattice-material-displacement-mesh.3mf");
    let reader = File::open(path).unwrap();

    let result = ThreemfPackage::from_reader_with_speed_optimized_deserializer(reader, true);

    match result {
        Ok(package) => {
            println!(
                "Number of build items: {}",
                get_model_view(&package.root).build_item_count()
            )
        }
        Err(err) => println!("Error reading the file: {:?}", err),
    }
}
