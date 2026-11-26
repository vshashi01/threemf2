use crate::model::domain::any_extensions::{AnyElementExtensions, AnyExtensions};
#[cfg(any(feature = "read", feature = "write"))]
use crate::model::domain::xml_struct::xml_struct;
use crate::{
    model::{OptionalResourceId, OptionalResourceIndex, ResourceIndex, StrResource},
    threemf_namespaces::{BEAM_LATTICE_BALLS_NS, BEAM_LATTICE_NS},
};

#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "read")]
use instant_xml::FromXml;

/// A beam lattice provides information about lattice data, in the form of a simplistic node-beam model
#[derive(Debug, PartialEq, Clone)]
pub struct BeamLattice {
    /// A producer MUST specify the minimal length of all beams in the lattice.
    /// The producer SHOULD NOT produce zero length beams (i.e. shorter than minlength).
    /// The consumer MUST ignore all beams with length shorter than minlength.
    pub minlength: f64,

    /// Default uniform radius value for the beams.
    pub radius: f64,

    /// Specifies whether balls are created at beam vertices
    pub ballmode: Option<BallMode>,

    /// Default uniform radius value for the balls. Required if ballmode is different to "none".
    pub ballradius: Option<f64>,

    /// Specifies the clipping mode of the beam lattice
    pub clippingmode: Option<ClippingMode>,

    /// References the clippingmesh object. Required if clippingmode is different to "none".
    pub clippingmesh: OptionalResourceId,

    /// References a mesh object that represents the intentional shape of the lattice geometry
    pub representationmesh: OptionalResourceId,

    /// Overrides the object-level pid as default for all beams
    pub pid: OptionalResourceId,

    /// Overrides the object-level pindex as default for all beams
    pub pindex: OptionalResourceIndex,

    /// Default capping mode for beam ends
    ///
    /// See [`CapMode`] for more details.
    pub cap: Option<CapMode>,

    /// Beams in this beam lattice.
    pub beams: Beams,

    /// Optional balls in this beam lattice.
    pub balls: Option<Balls>,

    /// Optional beam sets in this beam lattice.
    pub beamsets: Option<BeamSets>,

    /// Foreign attributes and child elements (`xs:anyAttribute` / `xs:any`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    BeamLattice {
        tag: "beamlattice",
        ns: BEAM_LATTICE_NS,
        prefixes: [("b2", BEAM_LATTICE_BALLS_NS)],
        force_prefix: false,
        attributes: {
            minlength: f64 => ("", "minlength"),
            radius: f64 => ("", "radius"),
            ballmode: Option<BallMode> => (BEAM_LATTICE_BALLS_NS, "ballmode"),
            ballradius: Option<f64> => (BEAM_LATTICE_BALLS_NS, "ballradius"),
            clippingmode: Option<ClippingMode> => (BEAM_LATTICE_NS, "clippingmode"),
            clippingmesh: OptionalResourceId => ("", "clippingmesh"),
            representationmesh: OptionalResourceId => ("", "representationmesh"),
            pid: OptionalResourceId => ("", "pid"),
            pindex: OptionalResourceIndex => ("", "pindex"),
            cap: Option<CapMode> => (BEAM_LATTICE_NS, "cap"),
        },
        elements: {
            beams: Beams => (BEAM_LATTICE_NS, "beams"),
            balls: Option<Balls> => (BEAM_LATTICE_BALLS_NS, "balls"),
            beamsets: Option<BeamSets> => (BEAM_LATTICE_NS, "beamsets"),
        },
        any: any_extensions: AnyExtensions,
    }
}

/// Ball mode for beam lattices - specifies whether balls are created at beam vertices
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
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
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
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
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, PartialEq, Eq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
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
#[derive(Debug, PartialEq, Clone)]
pub struct Beams {
    /// Field of beams
    pub beam: Vec<Beam>,

    /// Foreign child elements (`xs:any`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyElementExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    Beams {
        tag: "beams",
        ns: BEAM_LATTICE_NS,
        prefixes: [],
        force_prefix: false,
        attributes: {
        },
        elements: {
            beam: Vec<Beam> => (BEAM_LATTICE_NS, "beam"),
        },
        any: any_extensions: AnyElementExtensions,
    }
}

/// A single beam of the beamlattice
///
/// A beam is the core geometry within beam lattice. It has 2 vertices that defines
/// the end of the beam usually called beam nodes. Each beam nodes can have its own
/// thickness.
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
    xml(ns(BEAM_LATTICE_NS), rename = "beam")
)]
pub struct Beam {
    /// References a zero-based index into the vertices of this mesh. Defines the first vertex of the beam.
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub v1: ResourceIndex,

    /// References a zero-based index into the vertices of this mesh. Defines the second vertex of the beam.
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub v2: ResourceIndex,

    /// Defines the radius of the first vertex of beam. If not given, defaults to beamlattice radius.
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub r1: Option<f64>,

    /// Defines the radius of the second vertex of the beam. If not given, defaults to r1.
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub r2: Option<f64>,

    /// Overrides the beamlattice-level pindex for the first vertex of the beam
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub p1: OptionalResourceIndex,

    /// Overrides the beamlattice-level pindex for the second vertex of the beam
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub p2: OptionalResourceIndex,

    /// Overrides the beamlattice-level pid for the beam
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub pid: OptionalResourceId,

    /// Capping mode for the first end of the beam
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub cap1: Option<CapMode>,

    /// Capping mode for the second end of the beam
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub cap2: Option<CapMode>,
}

/// A Collection of Ball elements. See [`Ball`] for more details.
#[derive(Debug, PartialEq, Clone)]
pub struct Balls {
    /// Field of balls
    pub ball: Vec<Ball>,

    /// Foreign child elements (`xs:any`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyElementExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    Balls {
        tag: "balls",
        ns: BEAM_LATTICE_BALLS_NS,
        prefixes: [],
        force_prefix: true,
        attributes: {
        },
        elements: {
            ball: Vec<Ball> => (BEAM_LATTICE_BALLS_NS, "ball"),
        },
        any: any_extensions: AnyElementExtensions,
    }
}

/// A ball element defines a sphere of a given radius centered at the position of the vertex
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
    xml(ns(BEAM_LATTICE_BALLS_NS), rename = "ball", force_prefix)
)]
pub struct Ball {
    /// References a zero-based index into the vertices of this mesh.
    /// Defines the vertex that serves as the center for this ball.
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub vindex: ResourceIndex,

    /// The radius of this ball. If not given, uses default ballradius of the enclosing beamlattice.
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub r: Option<f64>,

    /// Overrides the beamlattice-level pindex for this ball
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub p: OptionalResourceIndex,

    /// Overrides the beamlattice-level pid for this ball
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub pid: OptionalResourceId,
}

/// A container for beam sets
#[derive(Debug, PartialEq, Clone)]
pub struct BeamSets {
    /// Field of beamsets
    pub beamset: Vec<BeamSet>,

    /// Foreign child elements (`xs:any`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyElementExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    BeamSets {
        tag: "beamsets",
        ns: BEAM_LATTICE_NS,
        prefixes: [],
        force_prefix: false,
        attributes: {
        },
        elements: {
            beamset: Vec<BeamSet> => (BEAM_LATTICE_NS, "beamset"),
        },
        any: any_extensions: AnyElementExtensions,
    }
}

/// A beam set contains a reference list to a subset of beams and a reference list to a subset of balls
#[derive(Debug, PartialEq, Clone)]
pub struct BeamSet {
    /// Human-readable name of the beam collection
    pub name: Option<StrResource>,

    /// Might be used for external identification of the beam collection data.
    /// The identifier attribute MUST be unique within the beam lattice.
    pub identifier: Option<StrResource>,

    /// References to beams in this set
    pub refs: Vec<BeamRef>,

    /// References to balls in this set  
    pub ballref: Vec<BallRef>,

    /// Foreign attributes and child elements (`xs:anyAttribute` / `xs:any`).
    ///
    /// Only populated with the `any` feature.
    pub any_extensions: AnyExtensions,
}

#[cfg(any(feature = "read", feature = "write"))]
xml_struct! {
    BeamSet {
        tag: "beamset",
        ns: BEAM_LATTICE_NS,
        prefixes: [],
        force_prefix: false,
        attributes: {
            name: Option<StrResource> => ("", "name"),
            identifier: Option<StrResource> => ("", "identifier"),
        },
        elements: {
            refs: Vec<BeamRef> => (BEAM_LATTICE_NS, "ref"),
            ballref: Vec<BallRef> => (BEAM_LATTICE_BALLS_NS, "ballref"),
        },
        any: any_extensions: AnyExtensions,
    }
}

/// A reference to a beam element
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
    xml(ns(BEAM_LATTICE_NS), rename = "ref")
)]
pub struct BeamRef {
    /// References an index in the beamlattice beam list
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
    pub index: ResourceIndex,
}

/// A reference to a ball element
#[cfg_attr(feature = "read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(
    any(feature = "write", feature = "read"),
    xml(ns(BEAM_LATTICE_BALLS_NS), rename = "ballref")
)]
pub struct BallRef {
    /// References an index in the beamlattice ball list
    #[cfg_attr(any(feature = "write", feature = "read"), xml(attribute))]
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
                any_extensions: Default::default(),
            },
            balls: None,
            beamsets: None,
            any_extensions: Default::default(),
        };
        let beamlattice_string = to_string(&beamlattice).unwrap();

        assert_eq!(beamlattice_string, xml_string);
    }

    #[test]
    pub fn toxml_beamlattice_with_balls_test() {
        let xml_string = format!(
            r#"<beamlattice xmlns="{}" xmlns:{}="{}" minlength="0.0001" radius="1" {}:ballmode="mixed" {}:ballradius="0.25" cap="sphere"><beams><beam v1="0" v2="1" r1="1.5" r2="1.6" /></beams><{}:balls><{}:ball vindex="0" r="0.5" /></{}:balls></beamlattice>"#,
            BEAM_LATTICE_NS,
            BEAM_LATTICE_BALLS_PREFIX,
            BEAM_LATTICE_BALLS_NS,
            BEAM_LATTICE_BALLS_PREFIX,
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
                any_extensions: Default::default(),
            },
            balls: Some(Balls {
                ball: vec![Ball {
                    vindex: 0,
                    r: Some(0.5),
                    p: OptionalResourceIndex::none(),
                    pid: OptionalResourceId::none(),
                }],
                any_extensions: Default::default(),
            }),
            beamsets: None,
            any_extensions: Default::default(),
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

#[cfg(feature = "read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::{
        model::OptionalResourceIndex,
        model::domain::{
            build::Build,
            mesh::{Mesh, Triangles, Vertex, Vertices},
            model::{Model, ThreemfExtensions},
            object::{Object, ObjectKind},
            resources::Resources,
        },
        threemf_namespaces::{
            BEAM_LATTICE_BALLS_NS, BEAM_LATTICE_BALLS_PREFIX, BEAM_LATTICE_NS, BEAM_LATTICE_PREFIX,
            CORE_NS, ThreemfNamespace,
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
                    any_extensions: Default::default(),
                },
                balls: None,
                beamsets: None,
                any_extensions: Default::default(),
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
                    any_extensions: Default::default(),
                },
                balls: Some(Balls {
                    ball: vec![Ball {
                        vindex: 0,
                        r: Some(0.5),
                        p: OptionalResourceIndex::none(),
                        pid: OptionalResourceId::none(),
                    }],
                    any_extensions: Default::default(),
                }),
                beamsets: None,
                any_extensions: Default::default(),
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
                    ],
                    any_extensions: Default::default(),
                },
                triangles: Triangles {
                    triangle: vec![],
                    any_extensions: Default::default()
                },
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
                        ],
                        any_extensions: Default::default(),
                    },
                    balls: None,
                    beamsets: None,
                    any_extensions: Default::default(),
                }),
                any_extensions: Default::default(),
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
                requiredextensions: ThreemfExtensions::new(&[
                    ThreemfNamespace::BeamLattice,
                    ThreemfNamespace::BeamLatticeBalls
                ]),
                recommendedextensions: ThreemfExtensions::default(),
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
                                ],
                                any_extensions: Default::default(),
                            },
                            triangles: Triangles {
                                triangle: vec![],
                                any_extensions: Default::default()
                            },
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
                                    ],
                                    any_extensions: Default::default(),
                                },
                                balls: Some(Balls {
                                    ball: vec![Ball {
                                        vindex: 0,
                                        r: Some(0.5),
                                        p: OptionalResourceIndex::none(),
                                        pid: OptionalResourceId::none()
                                    }],
                                    any_extensions: Default::default(),
                                }),
                                beamsets: None,
                                any_extensions: Default::default(),
                            }),
                            any_extensions: Default::default(),
                        })),
                        any_extensions: Default::default(),
                        metadatagroup: Default::default(),
                    }],
                    basematerials: vec![],
                    slicestack: vec![],
                    colorgroup: Vec::new(),
                    texture2dgroup: Vec::new(),
                    compositematerials: Vec::new(),
                    multiproperties: Vec::new(),
                    texture2d: Vec::new(),
                    displacement2d: Vec::new(),
                    normvectorgroup: Vec::new(),
                    disp2dgroup: Vec::new(),
                    any_extensions: Default::default(),
                },
                build: Build {
                    uuid: None,
                    item: vec![],
                    any_extensions: Default::default(),
                },
                any_extensions: Default::default(),
            }
        )
    }
}

#[cfg(test)]
mod from_string_tests {
    use super::{BallMode, CapMode, ClippingMode};

    #[test]
    fn ball_mode_from_string() {
        assert_eq!(BallMode::from("none".to_string()), BallMode::None);
        assert_eq!(BallMode::from("NONE".to_string()), BallMode::None);
        assert_eq!(BallMode::from("mixed".to_string()), BallMode::Mixed);
        assert_eq!(BallMode::from("all".to_string()), BallMode::All);
        assert_eq!(BallMode::from("unknown".to_string()), BallMode::None);
    }

    #[test]
    fn clipping_mode_from_string() {
        assert_eq!(ClippingMode::from("none".to_string()), ClippingMode::None);
        assert_eq!(
            ClippingMode::from("inside".to_string()),
            ClippingMode::Inside
        );
        assert_eq!(
            ClippingMode::from("outside".to_string()),
            ClippingMode::Outside
        );
        assert_eq!(
            ClippingMode::from("OUTSIDE".to_string()),
            ClippingMode::Outside
        );
        assert_eq!(
            ClippingMode::from("unknown".to_string()),
            ClippingMode::None
        );
    }

    #[test]
    fn cap_mode_from_string() {
        assert_eq!(CapMode::from("hemisphere".to_string()), CapMode::Hemisphere);
        assert_eq!(CapMode::from("sphere".to_string()), CapMode::Sphere);
        assert_eq!(CapMode::from("butt".to_string()), CapMode::Butt);
        assert_eq!(CapMode::from("BUTT".to_string()), CapMode::Butt);
        assert_eq!(CapMode::from("unknown".to_string()), CapMode::Sphere);
    }
}
