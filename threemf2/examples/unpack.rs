use threemf2::{
    model::{domain::model::Model, query::get_model_view},
    package::{CachePolicy, ThreemfPackageLazyReader},
};

use std::{fs::File, path::PathBuf};

/// This is an example showing unpacking the package and manually deserializing the root model
/// run with
/// `cargo run --example unpack --no-default-features --features package-lazy-read`
///
fn main() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/data/mesh-composedpart-separate-model-files.3mf");
    let reader = File::open(path).unwrap();

    let result = ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(
        reader,
        CachePolicy::NoCache,
    );

    match result {
        Ok(unpacked) => {
            let mut model: Option<Model> = None;
            //let model = serde_roxmltree::from_str::<Model>(&unpacked.root);
            let result_from_model = unpacked.with_model_xml(unpacked.root_model_path(), |xml| {
                let deserialized = instant_xml::from_str::<Model>(xml).unwrap();
                model = Some(deserialized);
            });
            assert!(result_from_model.is_ok());
            match model {
                Some(model) => println!(
                    "Number of build items: {}",
                    get_model_view(&model).build_item_count()
                ),
                None => println!("Error deserializing the model"),
            }
        }
        Err(err) => println!("Error reading the file: {:?}", err),
    }
}
