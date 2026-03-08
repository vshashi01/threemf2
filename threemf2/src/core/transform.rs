#[cfg(any(feature = "write", feature = "memory-optimized-read"))]
use instant_xml::{Error, Id};

#[cfg(feature = "write")]
use instant_xml::{Serializer, ToXml};

#[cfg(feature = "memory-optimized-read")]
use instant_xml::{Deserializer, FromXml, Kind};

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

use std::ops::Index;

const MATRIX_SIZE: usize = 12;

// for a matrix in 3mf
//
// | m00 m01 m02 0.0 |
// | m10 m11 m12 0.0 |
// | m20 m21 m22 0.0 |
// | m30 m31 m32 1.0 |
//
// the first 3 columns are represented as [m00, m01, m02, m10, m11, m12, m20, m21, m22, m30, m31, m32]
/// 4x3 transformation matrix for positioning and orienting 3D objects.
///
/// Represents a 3D affine transformation as a 12-element array in row-major order.
/// The matrix transforms points from object space to world space.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(from = "String"))]
#[derive(Debug, PartialEq, Clone)]
pub struct Transform(pub [f64; MATRIX_SIZE]);

#[cfg(feature = "write")]
impl ToXml for Transform {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        field: Option<Id<'_>>,
        serializer: &mut Serializer<W>,
    ) -> Result<(), Error> {
        let prefix = match field {
            Some(id) => {
                let prefix = serializer.write_start(id.name, id.ns)?;
                serializer.end_start()?;
                Some((prefix, id.name))
            }
            None => None,
        };

        let transform_str = self
            .0
            .iter()
            .map(|&m| format!("{:.6}", m))
            .collect::<Vec<String>>()
            .join(" ");
        serializer.write_str(&transform_str)?;

        if let Some((prefix, name)) = prefix {
            serializer.write_close(prefix, name)?;
        }

        Ok(())
    }
}

#[cfg(feature = "memory-optimized-read")]
impl<'xml> FromXml<'xml> for Transform {
    fn matches(id: Id<'_>, field: Option<Id<'_>>) -> bool {
        match field {
            Some(field) => id == field,
            None => false,
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut Deserializer<'cx, 'xml>,
    ) -> Result<(), Error> {
        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let value = match deserializer.take_str()? {
            Some(value) => value,
            None => return Err(Error::MissingValue("No transform string found")),
        };

        let result = Transform::from(value.into_owned());
        *into = Some(result);
        Ok(())
    }

    type Accumulator = Option<Self>;

    const KIND: Kind = Kind::Scalar;
}

#[cfg(not(feature = "memory-optimized-read-experimental"))]
impl From<String> for Transform {
    fn from(value: String) -> Self {
        let values = value
            .split(" ")
            .map(|v| v.parse::<f64>().unwrap_or_default())
            .collect::<Vec<f64>>();

        // write now it can always panic something to improve in the future
        Self(values.try_into().unwrap())
    }
}

#[cfg(feature = "memory-optimized-read-experimental")]
impl From<String> for Transform {
    fn from(value: String) -> Self {
        let values = value
            .split(" ")
            .map(|v| lexical::parse(v).unwrap_or_default())
            .collect::<Vec<f64>>();

        // write now it can always panic something to improve in the future
        Self(values.try_into().unwrap())
    }
}

impl Index<usize> for Transform {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        if index < MATRIX_SIZE {
            &self.0[index]
        } else {
            panic!("Unexpected index for Transform {:?}", index);
        }
    }
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::{ToXml, to_string};
    use pretty_assertions::assert_eq;

    use super::Transform;

    // Transform is a transparent tuple struct, it can only be properly write/read to/from XML when
    //placed in a separate struct
    #[derive(ToXml, PartialEq, Debug)]
    struct TestTransform {
        transform: Transform,
    }

    #[test]
    #[rustfmt::skip]
    fn toxml_test_transform() {
        let xml_string = "<TestTransform><transform>3.665893 -2718.281828 1618.033988 707.106781 -1414.213562 2236.067977 1442.249570 -866.025403 0.693556 1732.050807 -523.598775 577.215664</transform></TestTransform>";
       let test_transform = TestTransform{ transform: Transform([
            3.665893, -2718.281828, 1618.033988,
            707.106781, -1414.213562, 2236.067977,
            1442.249570, -866.025403, 0.693556,
            1732.050807, -523.598775, 577.215664,
        ])}; 
        let transform_string = to_string(&test_transform).unwrap();

        assert_eq!(transform_string, xml_string);
    }

    // Transform rename
    #[derive(ToXml, PartialEq, Debug)]
    #[xml(rename = "rename")]
    struct TestTransformRename {
        #[xml(rename = "transform-matrix")]
        transform: Transform,
    }

    #[test]
    #[rustfmt::skip]
    fn toxml_test_transform_rename() {
        let xml_string = "<rename><transform-matrix>4.141592 -2718.281828 1618.033988 707.106781 -1414.213562 2236.067977 1442.249570 -866.025403 0.793147 1732.050807 -523.598775 577.215664</transform-matrix></rename>";
        let test_transform = TestTransformRename {
            transform: Transform([
                4.141592, -2718.281828, 1618.033988,
                707.106781, -1414.213562, 2236.067977,
                1442.249570, -866.025403, 0.793147,
                1732.050807, -523.598775, 577.215664,
            ])
        };
        let transform_string = to_string(&test_transform).unwrap();

        assert_eq!(transform_string, xml_string);
    }
}

#[cfg(all(
    feature = "memory-optimized-read",
    not(feature = "memory-optimized-read-experimental")
))]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::{FromXml, from_str};
    use pretty_assertions::assert_eq;

    use super::Transform;

    // Transform is a transparent tuple struct, it can only be properly write/read to/from XML when
    //placed in a separate struct
    #[derive(FromXml, PartialEq, Debug)]
    struct TestTransform {
        transform: Transform,
    }

    #[test]
    #[rustfmt::skip]
    fn fromxml_test_transform() {
        let xml_string = "<TestTransform><transform>3.665893 -2718.281828 1618.033988 707.106781 -1414.213562 2236.067977 1442.249570 -866.025403 0.693556 1732.050807 -523.598775 577.215664</transform></TestTransform>";
        let test_transform = from_str::<TestTransform>(xml_string).unwrap();

        assert_eq!(
            test_transform.transform,
            Transform([
            3.665893, -2718.281828, 1618.033988,
            707.106781, -1414.213562, 2236.067977,
            1442.249570, -866.025403, 0.693556,
            1732.050807, -523.598775, 577.215664,
        ]));
    }

    // Transform rename
    #[derive(FromXml, PartialEq, Debug)]
    #[xml(rename = "rename")]
    struct TestTransformRename {
        #[xml(rename = "transform-matrix")]
        transform: Transform,
    }

    #[test]
    #[rustfmt::skip]
    fn fromxml_test_transform_rename() {
        let xml_string =
            "<rename><transform-matrix>4.141592 -2718.281828 1618.033988 707.106781 -1414.213562 2236.067977 1442.249570 -866.025403 0.793147 1732.050807 -523.598775 577.215664</transform-matrix></rename>";
        let test_transform = from_str::<TestTransformRename>(xml_string).unwrap();

        assert_eq!(
            test_transform.transform,
           Transform([
                4.141592, -2718.281828, 1618.033988,
                707.106781, -1414.213562, 2236.067977,
                1442.249570, -866.025403, 0.793147,
                1732.050807, -523.598775, 577.215664,
            ]) 
        );
    }
}

#[cfg(feature = "memory-optimized-read-experimental")]
#[cfg(test)]
mod memory_optimized_fast_float_read_tests {
    use instant_xml::{FromXml, from_str};
    use pretty_assertions::assert_eq;

    use super::Transform;

    // Transform is a transparent tuple struct, it can only be properly write/read to/from XML when
    //placed in a separate struct
    #[derive(FromXml, PartialEq, Debug)]
    struct TestTransform {
        transform: Transform,
    }

    #[test]
    #[rustfmt::skip]
    fn fromxml_test_transform() {
        let xml_string = "<TestTransform><transform>3.665893 -2718.281828 1618.033988 707.106781 -1414.213562 2236.067977 1442.249570 -866.025403 0.693556 1732.050807 -523.598775 577.215664</transform></TestTransform>";
        let test_transform = from_str::<TestTransform>(xml_string).unwrap();

        assert_eq!(
            test_transform.transform,
            Transform([
            3.665893, -2718.281828, 1618.033988,
            707.106781, -1414.213562, 2236.067977,
            1442.249570, -866.025403, 0.693556,
            1732.050807, -523.598775, 577.215664,
        ]));
    }

    // Transform rename
    #[derive(FromXml, PartialEq, Debug)]
    #[xml(rename = "rename")]
    struct TestTransformRename {
        #[xml(rename = "transform-matrix")]
        transform: Transform,
    }

    #[test]
    #[rustfmt::skip]
    fn fromxml_test_transform_rename() {
        let xml_string =
            "<rename><transform-matrix>4.141592 -2718.281828 1618.033988 707.106781 -1414.213562 2236.067977 1442.249570 -866.025403 0.793147 1732.050807 -523.598775 577.215664</transform-matrix></rename>";
        let test_transform = from_str::<TestTransformRename>(xml_string).unwrap();

        assert_eq!(
            test_transform.transform,
           Transform([
                4.141592, -2718.281828, 1618.033988,
                707.106781, -1414.213562, 2236.067977,
                1442.249570, -866.025403, 0.793147,
                1732.050807, -523.598775, 577.215664,
            ]) 
        );
    }
}

#[cfg(feature = "speed-optimized-read")]
#[cfg(test)]
mod speed_optimized_read_tests {
    use pretty_assertions::assert_eq;
    use serde::Deserialize;
    use serde_roxmltree::from_str;

    use super::Transform;

    // Transform is a transparent tuple struct, it can only be properly write/read to/from XML when
    // placed in a separate struct
    #[derive(Deserialize, PartialEq, Debug)]
    struct TestTransform {
        transform: Transform,
    }

    #[test]
    #[rustfmt::skip]
    fn fromxml_test_transform() {
        let xml_string = "<TestTransform><transform>3.665893 -2718.281828 1618.033988 707.106781 -1414.213562 2236.067977 1442.249570 -866.025403 0.693556 1732.050807 -523.598775 577.215664</transform></TestTransform>";
        let test_transform = from_str::<TestTransform>(xml_string).unwrap();

        assert_eq!(
            test_transform.transform,
            Transform([
            3.665893, -2718.281828, 1618.033988,
            707.106781, -1414.213562, 2236.067977,
            1442.249570, -866.025403, 0.693556,
            1732.050807, -523.598775, 577.215664,
        ]));
    }

    // Transform rename
    #[derive(Deserialize, PartialEq, Debug)]
    #[serde(rename = "rename")]
    struct TestTransformRename {
        #[serde(rename = "transform-matrix")]
        transform: Transform,
    }

    #[test]
    #[rustfmt::skip]
    fn fromxml_test_transform_rename() {
        let xml_string =
            "<rename><transform-matrix>4.141592 -2718.281828 1618.033988 707.106781 -1414.213562 2236.067977 1442.249570 -866.025403 0.793147 1732.050807 -523.598775 577.215664</transform-matrix></rename>";
        let test_transform = from_str::<TestTransformRename>(xml_string).unwrap();

        assert_eq!(
            test_transform.transform,
           Transform([
                4.141592, -2718.281828, 1618.033988,
                707.106781, -1414.213562, 2236.067977,
                1442.249570, -866.025403, 0.793147,
                1732.050807, -523.598775, 577.215664,
            ]) 
        );
    }
}
