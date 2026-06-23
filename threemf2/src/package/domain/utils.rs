//! XML utility functions for namespace parsing and manipulation.

use crate::package::domain::XmlNamespace;

/// Extracts xmlns attribute declarations from an XML element attribute definitions
pub fn parse_xmlns_attributes(tag_content: &str) -> Vec<XmlNamespace> {
    let mut attributes = Vec::new();
    let mut chars = tag_content.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == 'x' {
            // Check for "xmlns" or "xmlns:"
            let mut attr_name = String::from("x");
            let mut is_xmlns = true;

            // Read rest of potential "xmlns" or "xmlns:"
            for expected in ['m', 'l', 'n', 's'] {
                if chars.peek() == Some(&expected) {
                    attr_name.push(chars.next().unwrap());
                } else {
                    is_xmlns = false;
                    break;
                }
            }

            if is_xmlns {
                let prefix = if chars.peek() == Some(&':') {
                    chars.next(); // consume ':'
                    // Read namespace prefix
                    let mut prefix_str = String::new();
                    while let Some(&ch) = chars.peek() {
                        if ch.is_alphabetic() || ch == '_' || ch == '-' {
                            prefix_str.push(chars.next().unwrap());
                        } else {
                            break;
                        }
                    }
                    Some(prefix_str)
                } else {
                    None
                };

                // Expect equals sign
                if chars.next() == Some('=') && chars.next() == Some('"') {
                    let mut uri = String::new();
                    // Read until closing quote
                    for ch in chars.by_ref() {
                        if ch == '"' {
                            break;
                        }
                        uri.push(ch);
                    }
                    attributes.push(XmlNamespace { prefix, uri });
                }
            }
        }
    }

    attributes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xmlns_attributes_simple() {
        let xml = r#"<model xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02" unit="millimeter">"#;
        let attrs = parse_xmlns_attributes(xml);
        assert_eq!(
            attrs,
            vec![XmlNamespace {
                prefix: None,
                uri: "http://schemas.microsoft.com/3dmanufacturing/core/2015/02".to_string()
            }]
        );
    }

    #[test]
    fn test_parse_xmlns_attributes_prefixed() {
        let xml = r#"<model xmlns="http://core" xmlns:p="http://prod" xmlns:b="http://beam">"#;
        let attrs = parse_xmlns_attributes(xml);
        assert_eq!(
            attrs,
            vec![
                XmlNamespace {
                    prefix: None,
                    uri: "http://core".to_string()
                },
                XmlNamespace {
                    prefix: Some("p".to_string()),
                    uri: "http://prod".to_string()
                },
                XmlNamespace {
                    prefix: Some("b".to_string()),
                    uri: "http://beam".to_string()
                },
            ]
        );
    }

    #[test]
    fn test_parse_xmlns_attributes_empty() {
        let xml = r#"<model unit="millimeter">"#;
        let attrs = parse_xmlns_attributes(xml);
        assert_eq!(attrs, Vec::<XmlNamespace>::new());
    }

    #[test]
    fn test_parse_xmlns_attributes_mixed() {
        let xml = r#"<model xmlns="http://core" unit="millimeter" xmlns:p="http://prod" requiredextensions="ext">"#;
        let attrs = parse_xmlns_attributes(xml);
        assert_eq!(
            attrs,
            vec![
                XmlNamespace {
                    prefix: None,
                    uri: "http://core".to_string()
                },
                XmlNamespace {
                    prefix: Some("p".to_string()),
                    uri: "http://prod".to_string()
                },
            ]
        );
    }
}
