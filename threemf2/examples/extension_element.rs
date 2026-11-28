use instant_xml::{FromXml, ToXml};
use threemf2::model::{
    builder::{ModelBuilder, ObjectType, Unit},
    domain::any::AnyElement,
    query::get_objects_from_model,
};
use threemf2::package::{ThreemfPackage, ThreemfPackageBuilder};

use std::{fs::File, path::PathBuf};

/// Namespace of the custom extension. Foreign content must not use a 3MF namespace.
const ACME_NS: &str = "http://acme.example/3mf/labels/2026";

/// A custom extension element, modelled as a plain struct with the instant-xml derive.
/// The struct must use the same instant-xml version as threemf2.
#[derive(Debug, PartialEq, FromXml, ToXml)]
#[xml(ns("http://acme.example/3mf/labels/2026"), rename = "label")]
struct Label {
    #[xml(attribute)]
    code: u32,
    #[xml(attribute)]
    material: String,
    note: Vec<Note>,
}

#[derive(Debug, PartialEq, FromXml, ToXml)]
#[xml(ns("http://acme.example/3mf/labels/2026"), rename = "note")]
struct Note {
    #[xml(direct)]
    text: String,
}

/// Creates a custom extension element from a struct, adds it to an object and writes the
/// package to a 3MF file, then reads the file back and decodes the element again.
///
/// run with
/// `cargo run --example extension_element --features any -- [output.3mf]`
fn main() {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("extension_element.3mf"));

    let label = Label {
        code: 42,
        material: "PETG".to_owned(),
        note: vec![
            Note {
                text: "Print with brim".to_owned(),
            },
            Note {
                text: "Handle with care".to_owned(),
            },
        ],
    };

    // Struct -> AnyElement. The prefix hint is optional; it picks the prefix used in the file.
    let element = AnyElement::encode(&label)
        .expect("Error encoding the label")
        .with_prefix_hint("acme");

    let mut builder = ModelBuilder::new(Unit::Millimeter, true);
    builder.add_build(None).unwrap();

    let obj_id = builder
        .add_mesh_object(|obj| {
            obj.object_type(ObjectType::Model);
            obj.add_vertices(&[[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [0.0, 10.0, 0.0]]);
            obj.add_triangle(&[0, 1, 2]);
            // A foreign attribute and the foreign element on <object>.
            obj.any_attribute(ACME_NS, "batch", "A-17")?
                .any_element(element)?;
            Ok(())
        })
        .unwrap();
    builder.add_build_item(obj_id).unwrap();

    let mut package_builder = ThreemfPackageBuilder::new();
    package_builder.set_root_model(builder.build().unwrap());
    let package = package_builder.build().expect("Error building package");

    let file = File::create(&path).expect("Error creating the 3MF file");
    package
        .write(file)
        .expect("Error writing the model into a ThreemfPackage");
    println!("Wrote {}", path.display());

    // Read the file back and turn the foreign element into the struct again.
    let file = File::open(&path).expect("Error opening the 3MF file");
    let package = ThreemfPackage::from_reader(file, false).expect("Error reading the 3MF file");
    let object = get_objects_from_model(&package.root)
        .next()
        .expect("No object in the model");

    println!("acme:batch = {:?}", object.any_attribute(ACME_NS, "batch"));
    for element in object.any_elements_in(ACME_NS) {
        if element.is(ACME_NS, "label") {
            let read: Label = element.decode().expect("Error decoding the label");
            println!("{read:?}");
            assert_eq!(read, label);
        }
    }
}
