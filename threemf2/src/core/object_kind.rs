#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[derive(PartialEq, Debug, Clone)]
pub struct TestObject {
    pub id: u32,
    #[cfg_attr(feature = "speed-optimized-read", serde(rename = "#content", default))]
    pub kind: Option<SimpleKind>,
}

#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[derive(PartialEq, Debug, Clone)]
#[cfg_attr(feature = "speed-optimized-read", serde(rename_all = "lowercase"))]

pub enum SimpleKind {
    Entity(u8),
    Lala(f32),
}

#[test]
fn test_simple_entity_speed() {
    let obj = TestObject {
        id: 1,
        kind: Some(SimpleKind::Entity(5)),
    };
    let xml_string = r##"<TestObject id="1"><entity>5</entity></TestObject>"##;
    let de_obj = serde_roxmltree::from_str::<TestObject>(xml_string).unwrap();
    assert_eq!(obj, de_obj);
}

#[test]
fn test_simple_non_speed() {
    let obj = TestObject { id: 1, kind: None };
    let xml_string = r##"<TestObject id="1"></TestObject>"##;
    let de_obj = serde_roxmltree::from_str::<TestObject>(xml_string).unwrap();
    assert_eq!(obj, de_obj);
}
