//! XSD validation tests for 3MF files produced by threemf2
//!
//! This crate validates that XML output from threemf2 conforms to the official
//! 3MF Consortium XSD schemas.

// Validation utilities for XSD testing
pub mod validation {
    use fastxml::{
        parse,
        schema::{
            XmlSchemaValidationContext, parse_xsd_multiple, validate_document_by_schema_context,
        },
    };
    use std::io::{Cursor, Read};
    use zip::ZipArchive;

    /// Extract model XML from a 3MF package (ZIP archive)
    pub fn extract_model_xml(package_bytes: &[u8]) -> Result<String, String> {
        let cursor = Cursor::new(package_bytes);
        let mut archive =
            ZipArchive::new(cursor).map_err(|e| format!("Failed to open ZIP archive: {}", e))?;

        // Find and extract the 3D model file
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read ZIP entry {}: {}", i, e))?;

            let name = file.name();
            if name.ends_with(".model") || name.contains("3dmodel") {
                let mut content = String::new();
                file.read_to_string(&mut content)
                    .map_err(|e| format!("Failed to read model content: {}", e))?;
                return Ok(content);
            }
        }

        Err("No .model file found in 3MF package".to_string())
    }

    pub fn validate_against_xsd(xml: &str, contents: &[(&str, &[u8])]) -> Result<(), String> {
        let compiled_schema = parse_xsd_multiple(contents).unwrap();

        let context = XmlSchemaValidationContext::new(compiled_schema);

        let xml_doc = parse(xml).unwrap();

        match validate_document_by_schema_context(&xml_doc, &context) {
            Ok(result) => {
                if result.is_empty() {
                    Ok(())
                } else {
                    let errors: Vec<String> = result.iter().map(|e| e.to_string()).collect();
                    Err(errors.join("\n"))
                }
            }
            Err(err) => panic!("{err:?}"),
        }
    }

    /// Panic with detailed error message on validation failure
    pub fn validate_or_panic(xml: &str, contents: &[(&str, &[u8])], case_name: &str) {
        match validate_against_xsd(xml, contents) {
            Ok(()) => (),
            Err(e) => {
                panic!(
                    "XSD validation failed for {}:\n{}\n\nXML Content:\n{}",
                    case_name, e, xml
                );
            }
        }
    }
}
