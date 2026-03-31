use crate::{
    core::{OptionalResourceId, OptionalResourceIndex, ResourceIndex},
    threemf_namespaces::{BEAM_LATTICE_BALLS_NS, BEAM_LATTICE_NS},
};

#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "memory-optimized-read")]
use instant_xml::FromXml;

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

/// A beam lattice provides information about lattice data, in the form of a simplistic node-beam model
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(rename = "beamlattice"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(BEAM_LATTICE_NS, b2 = BEAM_LATTICE_BALLS_NS), rename = "beamlattice")
)]
pub struct BeamLattice {
    /// A producer MUST specify the minimal length of all beams in the lattice.
    /// The producer SHOULD NOT produce zero length beams (i.e. shorter than minlength).
    /// The consumer MUST ignore all beams with length shorter than minlength.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub minlength: f64,

    /// Default uniform radius value for the beams.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub radius: f64,

    /// Specifies whether balls are created at beam vertices
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(BEAM_LATTICE_BALLS_NS), attribute)
    )]
    pub ballmode: Option<BallMode>,

    /// Default uniform radius value for the balls. Required if ballmode is different to "none".
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(BEAM_LATTICE_BALLS_NS), attribute)
    )]
    pub ballradius: Option<f64>,

    /// Specifies the clipping mode of the beam lattice
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(BEAM_LATTICE_NS), attribute)
    )]
    pub clippingmode: Option<ClippingMode>,

    /// References the clippingmesh object. Required if clippingmode is different to "none".
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            default = "crate::core::types::serde_optional_resource_id::default_none",
            deserialize_with = "crate::core::types::serde_optional_resource_id::deserialize"
        )
    )]
    pub clippingmesh: OptionalResourceId,

    /// References a mesh object that represents the intentional shape of the lattice geometry
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            default = "crate::core::types::serde_optional_resource_id::default_none",
            deserialize_with = "crate::core::types::serde_optional_resource_id::deserialize"
        )
    )]
    pub representationmesh: OptionalResourceId,

    /// Overrides the object-level pid as default for all beams
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            default = "crate::core::types::serde_optional_resource_id::default_none",
            deserialize_with = "crate::core::types::serde_optional_resource_id::deserialize"
        )
    )]
    pub pid: OptionalResourceId,

    /// Overrides the object-level pindex as default for all beams
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            default = "crate::core::types::serde_impl::default_none",
            deserialize_with = "crate::core::types::serde_impl::deserialize"
        )
    )]
    pub pindex: OptionalResourceIndex,

    /// Default capping mode for beam ends
    ///
    /// See [`CapMode`] for more details.
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(BEAM_LATTICE_NS), attribute)
    )]
    pub cap: Option<CapMode>,

    /// Beams in this beam lattice.
    pub beams: Beams,

    /// Optional balls in this beam lattice.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(BEAM_LATTICE_BALLS_NS))
    )]
    pub balls: Option<Balls>,

    /// Optional beam sets in this beam lattice.
    pub beamsets: Option<BeamSets>,
}

/// Ball mode for beam lattices - specifies whether balls are created at beam vertices
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(from = "String"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(scalar, ns(BEAM_LATTICE_BALLS_NS), rename_all = "lowercase")
)]
pub enum BallMode {
    /// No balls are created at beam vertices
    #[default]
    None,

    /// Balls are created at vertices with a corresponding ball element specified in [`Ball`].
    /// Other vertices do not get a ball.
    Mixed,

    /// Balls are created at every vertex that maps to the end of a beam
    All,
}

impl From<String> for BallMode {
    fn from(value: String) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "none" => BallMode::None,
            "mixed" => BallMode::Mixed,
            "all" => BallMode::All,
            _ => BallMode::None,
        }
    }
}

/// Clipping mode for beam lattices
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(from = "String"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(scalar, rename_all = "lowercase")
)]
pub enum ClippingMode {
    /// The lattice is not clipped at any mesh boundary
    #[default]
    None,

    /// The lattice is clipped by the volume described by the referenced clippingmesh.
    /// All geometry inside the volume (according to the positive fill rule) is retained.
    Inside,

    /// The lattice is clipped by the volume described by the referenced clippingmesh.
    /// All geometry outside the volume (according to the positive fill rule) is retained.
    Outside,
}

impl From<String> for ClippingMode {
    fn from(value: String) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "none" => ClippingMode::None,
            "inside" => ClippingMode::Inside,
            "outside" => ClippingMode::Outside,
            _ => ClippingMode::None,
        }
    }
}

/// Capping mode for beam ends
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(from = "String"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(scalar, rename_all = "lowercase")
)]
pub enum CapMode {
    /// The beam end will be closed at its end nodes by a half sphere
    Hemisphere,

    /// The beam end will be closed at its end nodes by a sphere
    #[default]
    Sphere,

    /// The beam end will be closed with a flat end and therefore have a cylindrical or conical shape
    Butt,
}

impl From<String> for CapMode {
    fn from(value: String) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "hemisphere" => CapMode::Hemisphere,
            "sphere" => CapMode::Sphere,
            "butt" => CapMode::Butt,
            _ => CapMode::Sphere,
        }
    }
}

/// A container for beams
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(BEAM_LATTICE_NS), rename = "beams")
)]
pub struct Beams {
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub beam: Vec<Beam>,
}

/// A single beam of the beamlattice
///
/// A beam is the core geometry within beam lattice. It has 2 vertices that defines
/// the end of the beam usually called beam nodes. Each beam nodes can have its own
/// thickness.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(BEAM_LATTICE_NS), rename = "beam")
)]
pub struct Beam {
    /// References a zero-based index into the vertices of this mesh. Defines the first vertex of the beam.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub v1: ResourceIndex,

    /// References a zero-based index into the vertices of this mesh. Defines the second vertex of the beam.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub v2: ResourceIndex,

    /// Defines the radius of the first vertex of beam. If not given, defaults to beamlattice radius.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub r1: Option<f64>,

    /// Defines the radius of the second vertex of the beam. If not given, defaults to r1.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub r2: Option<f64>,

    /// Overrides the beamlattice-level pindex for the first vertex of the beam
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            default = "crate::core::types::serde_impl::default_none",
            deserialize_with = "crate::core::types::serde_impl::deserialize"
        )
    )]
    pub p1: OptionalResourceIndex,

    /// Overrides the beamlattice-level pindex for the second vertex of the beam
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            default = "crate::core::types::serde_impl::default_none",
            deserialize_with = "crate::core::types::serde_impl::deserialize"
        )
    )]
    pub p2: OptionalResourceIndex,

    /// Overrides the beamlattice-level pid for the beam
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            default = "crate::core::types::serde_optional_resource_id::default_none",
            deserialize_with = "crate::core::types::serde_optional_resource_id::deserialize"
        )
    )]
    pub pid: OptionalResourceId,

    /// Capping mode for the first end of the beam
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub cap1: Option<CapMode>,

    /// Capping mode for the second end of the beam
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub cap2: Option<CapMode>,
}

/// A Collection of Ball elements. See [`Ball`] for more details.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(BEAM_LATTICE_BALLS_NS), rename = "balls")
)]
pub struct Balls {
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub ball: Vec<Ball>,
}

/// A ball element defines a sphere of a given radius centered at the position of the vertex
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(BEAM_LATTICE_BALLS_NS), rename = "ball")
)]
pub struct Ball {
    /// References a zero-based index into the vertices of this mesh.
    /// Defines the vertex that serves as the center for this ball.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub vindex: ResourceIndex,

    /// The radius of this ball. If not given, uses default ballradius of the enclosing beamlattice.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub r: Option<f64>,

    /// Overrides the beamlattice-level pindex for this ball
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            default = "crate::core::types::serde_impl::default_none",
            deserialize_with = "crate::core::types::serde_impl::deserialize"
        )
    )]
    pub p: OptionalResourceIndex,

    /// Overrides the beamlattice-level pid for this ball
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    #[cfg_attr(
        feature = "speed-optimized-read",
        serde(
            default = "crate::core::types::serde_optional_resource_id::default_none",
            deserialize_with = "crate::core::types::serde_optional_resource_id::deserialize"
        )
    )]
    pub pid: OptionalResourceId,
}

/// A container for beam sets
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(BEAM_LATTICE_NS), rename = "beamsets")
)]
pub struct BeamSets {
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub beamset: Vec<BeamSet>,
}

/// A beam set contains a reference list to a subset of beams and a reference list to a subset of balls
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(BEAM_LATTICE_NS), rename = "beamset")
)]
pub struct BeamSet {
    /// Human-readable name of the beam collection
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub name: Option<String>,

    /// Might be used for external identification of the beam collection data.
    /// The identifier attribute MUST be unique within the beam lattice.
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub identifier: Option<String>,

    /// References to beams in this set
    #[cfg_attr(feature = "speed-optimized-read", serde(default, rename = "ref"))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(rename = "ref")
    )]
    pub refs: Vec<BeamRef>,

    /// References to balls in this set  
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(ns(BEAM_LATTICE_BALLS_NS))
    )]
    pub ballref: Vec<BallRef>,
}

/// A reference to a beam element
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(BEAM_LATTICE_NS), rename = "ref")
)]
pub struct BeamRef {
    /// References an index in the beamlattice beam list
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub index: ResourceIndex,
}

/// A reference to a ball element
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(ns(BEAM_LATTICE_BALLS_NS), rename = "ballref")
)]
pub struct BallRef {
    /// References an index in the beamlattice ball list
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub index: ResourceIndex,
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::to_string;
    use pretty_assertions::assert_eq;

    use crate::threemf_namespaces::{
        BEAM_LATTICE_BALLS_NS, BEAM_LATTICE_BALLS_PREFIX, BEAM_LATTICE_NS,
    };

    use super::*;

    #[test]
    pub fn toxml_beam_test() {
        let xml_string = format!(
            r#"<beam xmlns="{}" v1="0" v2="1" r1="1.5" r2="1.6" />"#,
            BEAM_LATTICE_NS
        );
        let beam = Beam {
            v1: 0,
            v2: 1,
            r1: Some(1.5),
            r2: Some(1.6),
            p1: OptionalResourceIndex::none(),
            p2: OptionalResourceIndex::none(),
            pid: OptionalResourceId::none(),
            cap1: None,
            cap2: None,
        };
        let beam_string = to_string(&beam).unwrap();

        assert_eq!(beam_string, xml_string);
    }

    #[test]
    pub fn toxml_ball_test() {
        let xml_string = format!(
            r#"<ball xmlns="{}" vindex="0" r="0.5" />"#,
            BEAM_LATTICE_BALLS_NS
        );
        let ball = Ball {
            vindex: 0,
            r: Some(0.5),
            p: OptionalResourceIndex::none(),
            pid: OptionalResourceId::none(),
        };
        let ball_string = to_string(&ball).unwrap();

        assert_eq!(ball_string, xml_string);
    }

    #[test]
    pub fn toxml_simple_beamlattice_test() {
        let xml_string = format!(
            r#"<beamlattice xmlns="{}" xmlns:{}="{}" minlength="0.0001" radius="1" cap="sphere"><beams><beam v1="0" v2="1" r1="1.5" r2="1.6" /></beams></beamlattice>"#,
            BEAM_LATTICE_NS, BEAM_LATTICE_BALLS_PREFIX, BEAM_LATTICE_BALLS_NS
        );
        let beamlattice = BeamLattice {
            minlength: 0.0001,
            radius: 1.0,
            ballmode: None,
            ballradius: None,
            clippingmode: None,
            clippingmesh: OptionalResourceId::none(),
            representationmesh: OptionalResourceId::none(),
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            cap: Some(CapMode::Sphere),
            beams: Beams {
                beam: vec![Beam {
                    v1: 0,
                    v2: 1,
                    r1: Some(1.5),
                    r2: Some(1.6),
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                    cap1: None,
                    cap2: None,
                }],
            },
            balls: None,
            beamsets: None,
        };
        let beamlattice_string = to_string(&beamlattice).unwrap();

        assert_eq!(beamlattice_string, xml_string);
    }

    #[test]
    pub fn toxml_beamlattice_with_balls_test() {
        let xml_string = format!(
            r#"<beamlattice xmlns="{}" xmlns:{}="{}" minlength="0.0001" radius="1" {}:ballmode="mixed" {}:ballradius="0.25" cap="sphere"><beams><beam v1="0" v2="1" r1="1.5" r2="1.6" /></beams><{}:balls><ball vindex="0" r="0.5" /></{}:balls></beamlattice>"#,
            BEAM_LATTICE_NS,
            BEAM_LATTICE_BALLS_PREFIX,
            BEAM_LATTICE_BALLS_NS,
            BEAM_LATTICE_BALLS_PREFIX,
            BEAM_LATTICE_BALLS_PREFIX,
            BEAM_LATTICE_BALLS_PREFIX,
            BEAM_LATTICE_BALLS_PREFIX,
        );
        let beamlattice = BeamLattice {
            minlength: 0.0001,
            radius: 1.0,
            ballmode: Some(BallMode::Mixed),
            ballradius: Some(0.25),
            clippingmode: None,
            clippingmesh: OptionalResourceId::none(),
            representationmesh: OptionalResourceId::none(),
            pid: OptionalResourceId::none(),
            pindex: OptionalResourceIndex::none(),
            cap: Some(CapMode::Sphere),
            beams: Beams {
                beam: vec![Beam {
                    v1: 0,
                    v2: 1,
                    r1: Some(1.5),
                    r2: Some(1.6),
                    p1: OptionalResourceIndex::none(),
                    p2: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                    cap1: None,
                    cap2: None,
                }],
            },
            balls: Some(Balls {
                ball: vec![Ball {
                    vindex: 0,
                    r: Some(0.5),
                    p: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                }],
            }),
            beamsets: None,
        };
        let beamlattice_string = to_string(&beamlattice).unwrap();

        assert_eq!(beamlattice_string, xml_string);
    }

    #[derive(Debug, ToXml, PartialEq, Eq)]
    #[xml(ns(b2 = BEAM_LATTICE_BALLS_NS))]
    struct EnumTestType {
        ballmode: Vec<BallMode>,

        clippingmode: Vec<ClippingMode>,

        capmode: Vec<CapMode>,
    }

    #[test]
    pub fn toxml_enums_test() {
        let xml_string = format!(
            r#"<EnumTestType xmlns:b2="{BEAM_LATTICE_BALLS_NS}"><b2:ballmode>none</b2:ballmode><b2:ballmode>mixed</b2:ballmode><b2:ballmode>all</b2:ballmode><clippingmode>none</clippingmode><clippingmode>inside</clippingmode><clippingmode>outside</clippingmode><capmode>hemisphere</capmode><capmode>sphere</capmode><capmode>butt</capmode></EnumTestType>"#
        );
        let enum_test = EnumTestType {
            ballmode: vec![BallMode::None, BallMode::Mixed, BallMode::All],
            clippingmode: vec![
                ClippingMode::None,
                ClippingMode::Inside,
                ClippingMode::Outside,
            ],
            capmode: vec![CapMode::Hemisphere, CapMode::Sphere, CapMode::Butt],
        };
        let enum_test_string = to_string(&enum_test).unwrap();

        assert_eq!(enum_test_string, xml_string);
    }
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            OptionalResourceIndex,
            build::Build,
            mesh::{Mesh, Triangles, Vertex, Vertices},
            model::Model,
            object::{Object, ObjectKind},
            resources::Resources,
        },
        threemf_namespaces::{
            BEAM_LATTICE_BALLS_NS, BEAM_LATTICE_BALLS_PREFIX, BEAM_LATTICE_NS, BEAM_LATTICE_PREFIX,
            CORE_NS,
        },
    };

    use super::*;

    #[test]
    pub fn fromxml_beam_test() {
        let xml_string = format!(
            r#"<beam xmlns="{}" v1="0" v2="1" r1="1.5" r2="1.6" />"#,
            BEAM_LATTICE_NS
        );
        let beam = from_str::<Beam>(&xml_string).unwrap();

        assert_eq!(
            beam,
            Beam {
                v1: 0,
                v2: 1,
                r1: Some(1.5),
                r2: Some(1.6),
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
                cap1: None,
                cap2: None,
            }
        );
    }

    #[test]
    pub fn fromxml_ball_test() {
        let xml_string = format!(
            r#"<ball xmlns="{}" vindex="0" r="0.5" />"#,
            BEAM_LATTICE_BALLS_NS
        );
        let ball = from_str::<Ball>(&xml_string).unwrap();

        assert_eq!(
            ball,
            Ball {
                vindex: 0,
                r: Some(0.5),
                p: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            }
        );
    }

    #[test]
    pub fn fromxml_simple_beamlattice_test() {
        let xml_string = format!(
            r#"<beamlattice xmlns="{}" xmlns:{}="{}" minlength="0.0001" radius="1" cap="sphere"><beams><beam v1="0" v2="1" r1="1.5" r2="1.6" /></beams></beamlattice>"#,
            BEAM_LATTICE_NS, BEAM_LATTICE_BALLS_PREFIX, BEAM_LATTICE_BALLS_NS
        );
        let beamlattice = from_str::<BeamLattice>(&xml_string).unwrap();

        assert_eq!(
            beamlattice,
            BeamLattice {
                minlength: 0.0001,
                radius: 1.0,
                ballmode: None,
                ballradius: None,
                clippingmode: None,
                clippingmesh: OptionalResourceId::none(),
                representationmesh: OptionalResourceId::none(),
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                cap: Some(CapMode::Sphere),
                beams: Beams {
                    beam: vec![Beam {
                        v1: 0,
                        v2: 1,
                        r1: Some(1.5),
                        r2: Some(1.6),
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                        cap1: None,
                        cap2: None,
                    }],
                },
                balls: None,
                beamsets: None,
            }
        );
    }

    #[test]
    pub fn fromxml_beamlattice_with_balls_test() {
        let xml_string = format!(
            r#"<beamlattice xmlns="{bl_ns}" xmlns:{bl2_prefix}="{bl2_ns}" minlength="0.0001" radius="1" cap="sphere" {bl2_prefix}:ballmode="mixed" {bl2_prefix}:ballradius="0.25"><beams><beam v1="0" v2="1" r1="1.5" r2="1.6" /></beams><{bl2_prefix}:balls><{bl2_prefix}:ball vindex="0" r="0.5" /></{bl2_prefix}:balls></beamlattice>"#,
            bl_ns = BEAM_LATTICE_NS,
            bl2_prefix = BEAM_LATTICE_BALLS_PREFIX,
            bl2_ns = BEAM_LATTICE_BALLS_NS,
        );
        let beamlattice = from_str::<BeamLattice>(&xml_string).unwrap();

        assert_eq!(
            beamlattice,
            BeamLattice {
                minlength: 0.0001,
                radius: 1.0,
                ballmode: Some(BallMode::Mixed),
                ballradius: Some(0.25),
                clippingmode: None,
                clippingmesh: OptionalResourceId::none(),
                representationmesh: OptionalResourceId::none(),
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                cap: Some(CapMode::Sphere),
                beams: Beams {
                    beam: vec![Beam {
                        v1: 0,
                        v2: 1,
                        r1: Some(1.5),
                        r2: Some(1.6),
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                        cap1: None,
                        cap2: None,
                    }],
                },
                balls: Some(Balls {
                    ball: vec![Ball {
                        vindex: 0,
                        r: Some(0.5),
                        p: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    }],
                }),
                beamsets: None,
            }
        );
    }

    #[derive(FromXml, Debug, PartialEq, Eq)]
    #[xml(ns(b2 = BEAM_LATTICE_BALLS_NS))]
    struct EnumTestType {
        ballmode: Vec<BallMode>,
        clippingmode: Vec<ClippingMode>,
        capmode: Vec<CapMode>,
    }

    #[test]
    pub fn fromxml_enums_test() {
        let xml_string = format!(
            r#"<EnumTestType xmlns:b2="{BEAM_LATTICE_BALLS_NS}"><b2:ballmode>none</b2:ballmode><b2:ballmode>mixed</b2:ballmode><b2:ballmode>all</b2:ballmode><clippingmode>none</clippingmode><clippingmode>inside</clippingmode><clippingmode>outside</clippingmode><capmode>hemisphere</capmode><capmode>sphere</capmode><capmode>butt</capmode></EnumTestType>"#
        );
        let enum_test = from_str::<EnumTestType>(&xml_string).unwrap();

        assert_eq!(
            enum_test,
            EnumTestType {
                ballmode: vec![BallMode::None, BallMode::Mixed, BallMode::All],
                clippingmode: vec![
                    ClippingMode::None,
                    ClippingMode::Inside,
                    ClippingMode::Outside
                ],
                capmode: vec![CapMode::Hemisphere, CapMode::Sphere, CapMode::Butt],
            }
        );
    }

    #[test]
    pub fn fromxml_mesh_with_beam_lattice_test() {
        let xml_string = format!(
            r##"<mesh xmlns="{}" xmlns:{}="{}"><vertices><vertex x="-1" y="-1" z="0" /><vertex x="1" y="-1" z="0" /><vertex x="1" y="1" z="0" /><vertex x="-1" y="1" z="0" /></vertices><triangles/><b:beamlattice minlength="0.0001" radius="0.25" cap="hemisphere"><b:beams><b:beam v1="0" v2="1" /><b:beam v1="1" v2="2" /><b:beam v1="0" v2="2" /></b:beams></b:beamlattice></mesh>"##,
            CORE_NS, BEAM_LATTICE_PREFIX, BEAM_LATTICE_NS
        );

        let mesh = from_str::<Mesh>(&xml_string).unwrap();

        assert_eq!(
            mesh,
            Mesh {
                vertices: Vertices {
                    vertex: vec![
                        Vertex::new(-1.0, -1.0, 0.0),
                        Vertex::new(1.0, -1.0, 0.0),
                        Vertex::new(1.0, 1.0, 0.0),
                        Vertex::new(-1.0, 1.0, 0.0),
                    ]
                },
                triangles: Triangles { triangle: vec![] },
                trianglesets: None,
                beamlattice: Some(BeamLattice {
                    minlength: 0.0001,
                    radius: 0.25,
                    ballmode: None,
                    ballradius: None,
                    clippingmode: None,
                    clippingmesh: OptionalResourceId::none(),
                    representationmesh: OptionalResourceId::none(),
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    cap: Some(CapMode::Hemisphere),
                    beams: Beams {
                        beam: vec![
                            Beam {
                                v1: 0,
                                v2: 1,
                                r1: None,
                                r2: None,
                                p1: OptionalResourceIndex::none(),
                                p2: OptionalResourceIndex::none(),
                                pid: OptionalResourceId::none(),
                                cap1: None,
                                cap2: None
                            },
                            Beam {
                                v1: 1,
                                v2: 2,
                                r1: None,
                                r2: None,
                                p1: OptionalResourceIndex::none(),
                                p2: OptionalResourceIndex::none(),
                                pid: OptionalResourceId::none(),
                                cap1: None,
                                cap2: None
                            },
                            Beam {
                                v1: 0,
                                v2: 2,
                                r1: None,
                                r2: None,
                                p1: OptionalResourceIndex::none(),
                                p2: OptionalResourceIndex::none(),
                                pid: OptionalResourceId::none(),
                                cap1: None,
                                cap2: None
                            }
                        ]
                    },
                    balls: None,
                    beamsets: None,
                }),
            }
        )
    }

    #[test]
    pub fn fromxml_model_with_beam_lattice_with_balls_test() {
        let xml_string = format!(
            r##"<model xmlns="{core_ns}" xmlns:{bl_prefix}="{bl_ns}" xmlns:{bl2_prefix}="{bl2_ns}" requiredextensions="b b2"><resources><object id="1"><mesh><vertices><vertex x="-1" y="-1" z="0" /><vertex x="1" y="-1" z="0" /><vertex x="1" y="1" z="0" /><vertex x="-1" y="1" z="0" /></vertices><triangles/><{bl_prefix}:beamlattice minlength="0.0001" radius="0.25" cap="hemisphere" {bl2_prefix}:ballmode="mixed" {bl2_prefix}:ballradius="0.25"><{bl_prefix}:beams><b:beam v1="0" v2="1" /><{bl_prefix}:beam v1="1" v2="2" /><{bl_prefix}:beam v1="0" v2="2" /></{bl_prefix}:beams><{bl2_prefix}:balls><{bl2_prefix}:ball vindex="0" r="0.5" /></{bl2_prefix}:balls></{bl_prefix}:beamlattice></mesh></object></resources><build></build></model>"##,
            core_ns = CORE_NS,
            bl_prefix = BEAM_LATTICE_PREFIX,
            bl_ns = BEAM_LATTICE_NS,
            bl2_prefix = BEAM_LATTICE_BALLS_PREFIX,
            bl2_ns = BEAM_LATTICE_BALLS_NS
        );

        let mesh = from_str::<Model>(&xml_string).unwrap();

        assert_eq!(
            mesh,
            Model {
                unit: None,
                requiredextensions: Some("b b2".to_owned()),
                recommendedextensions: None,
                metadata: vec![],
                resources: Resources {
                    object: vec![Object {
                        id: 1,
                        objecttype: None,
                        thumbnail: None,
                        partnumber: None,
                        name: None,
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                        meshresolution: None,
                        kind: Some(ObjectKind::Mesh(Mesh {
                            vertices: Vertices {
                                vertex: vec![
                                    Vertex::new(-1.0, -1.0, 0.0),
                                    Vertex::new(1.0, -1.0, 0.0),
                                    Vertex::new(1.0, 1.0, 0.0),
                                    Vertex::new(-1.0, 1.0, 0.0),
                                ]
                            },
                            triangles: Triangles { triangle: vec![] },
                            trianglesets: None,
                            beamlattice: Some(BeamLattice {
                                minlength: 0.0001,
                                radius: 0.25,
                                ballmode: Some(BallMode::Mixed),
                                ballradius: Some(0.25),
                                clippingmode: None,
                                clippingmesh: OptionalResourceId::none(),
                                representationmesh: OptionalResourceId::none(),
                                pid: OptionalResourceId::none(),
                                pindex: OptionalResourceIndex::none(),
                                cap: Some(CapMode::Hemisphere),
                                beams: Beams {
                                    beam: vec![
                                        Beam {
                                            v1: 0,
                                            v2: 1,
                                            r1: None,
                                            r2: None,
                                            p1: OptionalResourceIndex::none(),
                                            p2: OptionalResourceIndex::none(),
                                            pid: OptionalResourceId::none(),
                                            cap1: None,
                                            cap2: None
                                        },
                                        Beam {
                                            v1: 1,
                                            v2: 2,
                                            r1: None,
                                            r2: None,
                                            p1: OptionalResourceIndex::none(),
                                            p2: OptionalResourceIndex::none(),
                                            pid: OptionalResourceId::none(),
                                            cap1: None,
                                            cap2: None
                                        },
                                        Beam {
                                            v1: 0,
                                            v2: 2,
                                            r1: None,
                                            r2: None,
                                            p1: OptionalResourceIndex::none(),
                                            p2: OptionalResourceIndex::none(),
                                            pid: OptionalResourceId::none(),
                                            cap1: None,
                                            cap2: None
                                        }
                                    ]
                                },
                                balls: Some(Balls {
                                    ball: vec![Ball {
                                        vindex: 0,
                                        r: Some(0.5),
                                        p: OptionalResourceIndex::none(),
                                        pid: OptionalResourceId::none()
                                    }]
                                }),
                                beamsets: None,
                            }),
                        })),
                    }],
                    basematerials: vec![],
                    slicestack: vec![],
                },
                build: Build {
                    uuid: None,
                    item: vec![]
                }
            }
        )
    }
}

#[cfg(feature = "speed-optimized-read")]
#[cfg(test)]
mod speed_optimized_read_tests {
    use pretty_assertions::assert_eq;
    use serde_roxmltree::from_str;

    use crate::{
        core::{
            OptionalResourceIndex,
            build::Build,
            mesh::{Mesh, Triangles, Vertex, Vertices},
            model::Model,
            object::{Object, ObjectKind},
            resources::Resources,
        },
        threemf_namespaces::{
            BEAM_LATTICE_BALLS_NS, BEAM_LATTICE_BALLS_PREFIX, BEAM_LATTICE_NS, BEAM_LATTICE_PREFIX,
            CORE_NS,
        },
    };

    use super::*;

    #[test]
    pub fn fromxml_beam_test() {
        let xml_string = format!(
            r#"<beam xmlns="{}" v1="0" v2="1" r1="1.5" r2="1.6" />"#,
            BEAM_LATTICE_NS
        );
        let beam = from_str::<Beam>(&xml_string).unwrap();

        assert_eq!(
            beam,
            Beam {
                v1: 0,
                v2: 1,
                r1: Some(1.5),
                r2: Some(1.6),
                p1: OptionalResourceIndex::none(),
                p2: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
                cap1: None,
                cap2: None,
            }
        );
    }

    #[test]
    pub fn fromxml_ball_test() {
        let xml_string = format!(
            r#"<ball xmlns="{}" vindex="0" r="0.5" />"#,
            BEAM_LATTICE_BALLS_NS
        );
        let ball = from_str::<Ball>(&xml_string).unwrap();

        assert_eq!(
            ball,
            Ball {
                vindex: 0,
                r: Some(0.5),
                p: OptionalResourceIndex::none(),
                pid: OptionalResourceId::none(),
            }
        );
    }

    #[test]
    pub fn fromxml_simple_beamlattice_test() {
        let xml_string = format!(
            r#"<beamlattice xmlns="{}" xmlns:{}="{}" minlength="0.0001" radius="1" cap="sphere"><beams><beam v1="0" v2="1" r1="1.5" r2="1.6" /></beams></beamlattice>"#,
            BEAM_LATTICE_NS, BEAM_LATTICE_BALLS_PREFIX, BEAM_LATTICE_BALLS_NS
        );
        let beamlattice = from_str::<BeamLattice>(&xml_string).unwrap();

        assert_eq!(
            beamlattice,
            BeamLattice {
                minlength: 0.0001,
                radius: 1.0,
                ballmode: None,
                ballradius: None,
                clippingmode: None,
                clippingmesh: OptionalResourceId::none(),
                representationmesh: OptionalResourceId::none(),
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                cap: Some(CapMode::Sphere),
                beams: Beams {
                    beam: vec![Beam {
                        v1: 0,
                        v2: 1,
                        r1: Some(1.5),
                        r2: Some(1.6),
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                        cap1: None,
                        cap2: None,
                    }],
                },
                balls: None,
                beamsets: None,
            }
        );
    }

    #[test]
    pub fn fromxml_beamlattice_with_balls_test() {
        let xml_string = format!(
            r#"<beamlattice xmlns="{}" xmlns:{}="{}" minlength="0.0001" radius="1" cap="sphere" {}:ballmode="mixed" {}:ballradius="0.25"><beams><beam v1="0" v2="1" r1="1.5" r2="1.6" /></beams><{}:balls><{}:ball vindex="0" r="0.5" /></{}:balls></beamlattice>"#,
            BEAM_LATTICE_NS,
            BEAM_LATTICE_BALLS_PREFIX,
            BEAM_LATTICE_BALLS_NS,
            BEAM_LATTICE_BALLS_PREFIX,
            BEAM_LATTICE_BALLS_PREFIX,
            BEAM_LATTICE_BALLS_PREFIX,
            BEAM_LATTICE_BALLS_PREFIX,
            BEAM_LATTICE_BALLS_PREFIX
        );
        let beamlattice = from_str::<BeamLattice>(&xml_string).unwrap();

        assert_eq!(
            beamlattice,
            BeamLattice {
                minlength: 0.0001,
                radius: 1.0,
                ballmode: Some(BallMode::Mixed),
                ballradius: Some(0.25),
                clippingmode: None,
                clippingmesh: OptionalResourceId::none(),
                representationmesh: OptionalResourceId::none(),
                pid: OptionalResourceId::none(),
                pindex: OptionalResourceIndex::none(),
                cap: Some(CapMode::Sphere),
                beams: Beams {
                    beam: vec![Beam {
                        v1: 0,
                        v2: 1,
                        r1: Some(1.5),
                        r2: Some(1.6),
                        p1: OptionalResourceIndex::none(),
                        p2: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                        cap1: None,
                        cap2: None,
                    }],
                },
                balls: Some(Balls {
                    ball: vec![Ball {
                        vindex: 0,
                        r: Some(0.5),
                        p: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    }],
                }),
                beamsets: None,
            }
        );
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct EnumTestType {
        ballmode: Vec<BallMode>,
        clippingmode: Vec<ClippingMode>,
        capmode: Vec<CapMode>,
    }

    #[test]
    pub fn fromxml_enums_test() {
        let xml_string = "<EnumTestType><ballmode>none</ballmode><ballmode>mixed</ballmode><ballmode>all</ballmode><clippingmode>none</clippingmode><clippingmode>inside</clippingmode><clippingmode>outside</clippingmode><capmode>hemisphere</capmode><capmode>sphere</capmode><capmode>butt</capmode></EnumTestType>";
        let enum_test = from_str::<EnumTestType>(xml_string).unwrap();

        assert_eq!(
            enum_test,
            EnumTestType {
                ballmode: vec![BallMode::None, BallMode::Mixed, BallMode::All],
                clippingmode: vec![
                    ClippingMode::None,
                    ClippingMode::Inside,
                    ClippingMode::Outside
                ],
                capmode: vec![CapMode::Hemisphere, CapMode::Sphere, CapMode::Butt],
            }
        );
    }

    #[test]
    pub fn fromxml_mesh_with_beam_lattice_test() {
        let xml_string = format!(
            r##"<mesh xmlns="{}" xmlns:{}="{}"><vertices><vertex x="-1" y="-1" z="0" /><vertex x="1" y="-1" z="0" /><vertex x="1" y="1" z="0" /><vertex x="-1" y="1" z="0" /></vertices><triangles/><b:beamlattice minlength="0.0001" radius="0.25" cap="hemisphere"><b:beams><b:beam v1="0" v2="1" /><b:beam v1="1" v2="2" /><b:beam v1="0" v2="2" /></b:beams></b:beamlattice></mesh>"##,
            CORE_NS, BEAM_LATTICE_PREFIX, BEAM_LATTICE_NS
        );

        let mesh = from_str::<Mesh>(&xml_string).unwrap();

        assert_eq!(
            mesh,
            Mesh {
                vertices: Vertices {
                    vertex: vec![
                        Vertex::new(-1.0, -1.0, 0.0),
                        Vertex::new(1.0, -1.0, 0.0),
                        Vertex::new(1.0, 1.0, 0.0),
                        Vertex::new(-1.0, 1.0, 0.0),
                    ]
                },
                triangles: Triangles { triangle: vec![] },
                trianglesets: None,
                beamlattice: Some(BeamLattice {
                    minlength: 0.0001,
                    radius: 0.25,
                    ballmode: None,
                    ballradius: None,
                    clippingmode: None,
                    clippingmesh: OptionalResourceId::none(),
                    representationmesh: OptionalResourceId::none(),
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    cap: Some(CapMode::Hemisphere),
                    beams: Beams {
                        beam: vec![
                            Beam {
                                v1: 0,
                                v2: 1,
                                r1: None,
                                r2: None,
                                p1: OptionalResourceIndex::none(),
                                p2: OptionalResourceIndex::none(),
                                pid: OptionalResourceId::none(),
                                cap1: None,
                                cap2: None
                            },
                            Beam {
                                v1: 1,
                                v2: 2,
                                r1: None,
                                r2: None,
                                p1: OptionalResourceIndex::none(),
                                p2: OptionalResourceIndex::none(),
                                pid: OptionalResourceId::none(),
                                cap1: None,
                                cap2: None
                            },
                            Beam {
                                v1: 0,
                                v2: 2,
                                r1: None,
                                r2: None,
                                p1: OptionalResourceIndex::none(),
                                p2: OptionalResourceIndex::none(),
                                pid: OptionalResourceId::none(),
                                cap1: None,
                                cap2: None
                            }
                        ]
                    },
                    balls: None,
                    beamsets: None,
                }),
            }
        )
    }

    #[test]
    pub fn fromxml_mesh_with_beam_lattice_with_balls_test() {
        let xml_string = format!(
            r##"<mesh xmlns="{core_ns}" xmlns:{bl_prefix}="{bl_ns}" xmlns:{bl2_prefix}="{bl2_ns}"><vertices><vertex x="-1" y="-1" z="0" /><vertex x="1" y="-1" z="0" /><vertex x="1" y="1" z="0" /><vertex x="-1" y="1" z="0" /></vertices><triangles/><b:beamlattice minlength="0.0001" radius="0.25" cap="hemisphere" {bl2_prefix}:ballmode="mixed" {bl2_prefix}:ballradius="0.25"><b:beams><b:beam v1="0" v2="1" /><b:beam v1="1" v2="2" /><b:beam v1="0" v2="2" /></b:beams><{bl2_prefix}:balls><{bl2_prefix}:ball vindex="0" r="0.5" /></{bl2_prefix}:balls></b:beamlattice></mesh>"##,
            core_ns = CORE_NS,
            bl_prefix = BEAM_LATTICE_PREFIX,
            bl_ns = BEAM_LATTICE_NS,
            bl2_prefix = BEAM_LATTICE_BALLS_PREFIX,
            bl2_ns = BEAM_LATTICE_BALLS_NS
        );

        let mesh = from_str::<Mesh>(&xml_string).unwrap();

        assert_eq!(
            mesh,
            Mesh {
                vertices: Vertices {
                    vertex: vec![
                        Vertex::new(-1.0, -1.0, 0.0),
                        Vertex::new(1.0, -1.0, 0.0),
                        Vertex::new(1.0, 1.0, 0.0),
                        Vertex::new(-1.0, 1.0, 0.0),
                    ]
                },
                triangles: Triangles { triangle: vec![] },
                trianglesets: None,
                beamlattice: Some(BeamLattice {
                    minlength: 0.0001,
                    radius: 0.25,
                    ballmode: Some(BallMode::Mixed),
                    ballradius: Some(0.25),
                    clippingmode: None,
                    clippingmesh: OptionalResourceId::none(),
                    representationmesh: OptionalResourceId::none(),
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    cap: Some(CapMode::Hemisphere),
                    beams: Beams {
                        beam: vec![
                            Beam {
                                v1: 0,
                                v2: 1,
                                r1: None,
                                r2: None,
                                p1: OptionalResourceIndex::none(),
                                p2: OptionalResourceIndex::none(),
                                pid: OptionalResourceId::none(),
                                cap1: None,
                                cap2: None
                            },
                            Beam {
                                v1: 1,
                                v2: 2,
                                r1: None,
                                r2: None,
                                p1: OptionalResourceIndex::none(),
                                p2: OptionalResourceIndex::none(),
                                pid: OptionalResourceId::none(),
                                cap1: None,
                                cap2: None
                            },
                            Beam {
                                v1: 0,
                                v2: 2,
                                r1: None,
                                r2: None,
                                p1: OptionalResourceIndex::none(),
                                p2: OptionalResourceIndex::none(),
                                pid: OptionalResourceId::none(),
                                cap1: None,
                                cap2: None
                            }
                        ]
                    },
                    balls: Some(Balls {
                        ball: vec![Ball {
                            vindex: 0,
                            r: Some(0.5),
                            p: OptionalResourceIndex::none(),
                            pid: OptionalResourceId::none()
                        }]
                    }),
                    beamsets: None,
                }),
            }
        )
    }

    #[test]
    pub fn fromxml_model_with_beam_lattice_with_balls_test() {
        let xml_string = format!(
            r##"<model xmlns="{core_ns}" xmlns:{bl_prefix}="{bl_ns}" xmlns:{bl2_prefix}="{bl2_ns}" requiredextensions="b b2"><resources><object id="1"><mesh><vertices><vertex x="-1" y="-1" z="0" /><vertex x="1" y="-1" z="0" /><vertex x="1" y="1" z="0" /><vertex x="-1" y="1" z="0" /></vertices><triangles/><{bl_prefix}:beamlattice minlength="0.0001" radius="0.25" cap="hemisphere" {bl2_prefix}:ballmode="mixed" {bl2_prefix}:ballradius="0.25"><{bl_prefix}:beams><b:beam v1="0" v2="1" /><{bl_prefix}:beam v1="1" v2="2" /><{bl_prefix}:beam v1="0" v2="2" /></{bl_prefix}:beams><{bl2_prefix}:balls><{bl2_prefix}:ball vindex="0" r="0.5" /></{bl2_prefix}:balls></{bl_prefix}:beamlattice></mesh></object></resources><build></build></model>"##,
            core_ns = CORE_NS,
            bl_prefix = BEAM_LATTICE_PREFIX,
            bl_ns = BEAM_LATTICE_NS,
            bl2_prefix = BEAM_LATTICE_BALLS_PREFIX,
            bl2_ns = BEAM_LATTICE_BALLS_NS
        );

        let mesh = from_str::<Model>(&xml_string).unwrap();

        assert_eq!(
            mesh,
            Model {
                unit: None,
                requiredextensions: Some("b b2".to_owned()),
                recommendedextensions: None,
                metadata: vec![],
                resources: Resources {
                    object: vec![Object {
                        id: 1,
                        objecttype: None,
                        thumbnail: None,
                        partnumber: None,
                        name: None,
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        slicestackid: OptionalResourceId::none(),
                        slicepath: None,
                        meshresolution: None,
                        kind: Some(ObjectKind::Mesh(Mesh {
                            vertices: Vertices {
                                vertex: vec![
                                    Vertex::new(-1.0, -1.0, 0.0),
                                    Vertex::new(1.0, -1.0, 0.0),
                                    Vertex::new(1.0, 1.0, 0.0),
                                    Vertex::new(-1.0, 1.0, 0.0),
                                ]
                            },
                            triangles: Triangles { triangle: vec![] },
                            trianglesets: None,
                            beamlattice: Some(BeamLattice {
                                minlength: 0.0001,
                                radius: 0.25,
                                ballmode: Some(BallMode::Mixed),
                                ballradius: Some(0.25),
                                clippingmode: None,
                                clippingmesh: OptionalResourceId::none(),
                                representationmesh: OptionalResourceId::none(),
                                pid: OptionalResourceId::none(),
                                pindex: OptionalResourceIndex::none(),
                                cap: Some(CapMode::Hemisphere),
                                beams: Beams {
                                    beam: vec![
                                        Beam {
                                            v1: 0,
                                            v2: 1,
                                            r1: None,
                                            r2: None,
                                            p1: OptionalResourceIndex::none(),
                                            p2: OptionalResourceIndex::none(),
                                            pid: OptionalResourceId::none(),
                                            cap1: None,
                                            cap2: None
                                        },
                                        Beam {
                                            v1: 1,
                                            v2: 2,
                                            r1: None,
                                            r2: None,
                                            p1: OptionalResourceIndex::none(),
                                            p2: OptionalResourceIndex::none(),
                                            pid: OptionalResourceId::none(),
                                            cap1: None,
                                            cap2: None
                                        },
                                        Beam {
                                            v1: 0,
                                            v2: 2,
                                            r1: None,
                                            r2: None,
                                            p1: OptionalResourceIndex::none(),
                                            p2: OptionalResourceIndex::none(),
                                            pid: OptionalResourceId::none(),
                                            cap1: None,
                                            cap2: None
                                        }
                                    ]
                                },
                                balls: Some(Balls {
                                    ball: vec![Ball {
                                        vindex: 0,
                                        r: Some(0.5),
                                        p: OptionalResourceIndex::none(),
                                        pid: OptionalResourceId::none()
                                    }]
                                }),
                                beamsets: None,
                            }),
                        })),
                    }],
                    basematerials: vec![],
                    slicestack: vec![],
                },
                build: Build {
                    uuid: None,
                    item: vec![]
                }
            }
        )
    }
}
