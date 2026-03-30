#[cfg(feature = "write")]
use instant_xml::ToXml;

#[cfg(feature = "memory-optimized-read")]
use instant_xml::FromXml;

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

#[cfg(feature = "write")]
use crate::{
    core::{build::Build, metadata::Metadata, object::ObjectKind, resources::Resources},
    threemf_namespaces::{
        BEAM_LATTICE_NS, BOOLEAN_NS, CORE_NS, CORE_TRIANGLESET_NS, PROD_NS, ThreemfNamespace,
    },
};

/// Represents a 3MF model, the root element containing resources and build configuration.
///
/// A model defines the 3D objects, materials, and build instructions for a 3MF package.
/// It serves as the primary container for all 3MF data structures.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(rename = "model"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(any(feature="write", feature="memory-optimized-read"), xml(ns(CORE_NS, p = PROD_NS, t = CORE_TRIANGLESET_NS, b = BEAM_LATTICE_NS, bo = BOOLEAN_NS), rename = "model"))]
pub struct Model {
    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub unit: Option<Unit>,

    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub requiredextensions: Option<String>,

    #[cfg_attr(
        any(feature = "write", feature = "memory-optimized-read"),
        xml(attribute)
    )]
    pub recommendedextensions: Option<String>,

    #[cfg_attr(feature = "speed-optimized-read", serde(default))]
    pub metadata: Vec<Metadata>,

    pub resources: Resources,

    pub build: Build,
}

/// Model measurement unit, default is millimeter
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(from = "String"))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    any(feature = "write", feature = "memory-optimized-read"),
    xml(scalar, rename_all = "lowercase")
)]
pub enum Unit {
    Micron,
    #[default]
    Millimeter,
    Centimeter,
    Inch,
    Foot,
    Meter,
}

impl From<String> for Unit {
    fn from(value: String) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "micron" => Unit::Micron,
            "millimeter" => Unit::Millimeter,
            "centimeter" => Unit::Centimeter,
            "inch" => Unit::Inch,
            "foot" => Unit::Foot,
            "meter" => Unit::Meter,
            _ => Unit::Millimeter,
        }
    }
}

#[cfg(feature = "write")]
impl Model {
    pub fn used_namespaces(&self) -> Vec<ThreemfNamespace> {
        let mut used = vec![ThreemfNamespace::Core];

        if self.uses_prod_ns() {
            used.push(ThreemfNamespace::Prod);
        }

        if self.uses_beamlattice_ns() {
            used.push(ThreemfNamespace::BeamLattice);
        }

        if self.uses_triangleset_ns() {
            used.push(ThreemfNamespace::CoreTriangleSet);
        }

        if self.uses_boolean_ns() {
            used.push(ThreemfNamespace::Boolean);
        }

        used
    }

    fn uses_prod_ns(&self) -> bool {
        if self.build.uuid.is_some() {
            return true;
        }

        for item in &self.build.item {
            if item.path.is_some() || item.uuid.is_some() {
                return true;
            }
        }

        for obj in &self.resources.object {
            if obj.uuid.is_some() {
                return true;
            }

            if let Some(kind) = &obj.kind
                && let ObjectKind::Components(comps) = kind
            {
                for comp in &comps.component {
                    if comp.path.is_some() || comp.uuid.is_some() {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn uses_beamlattice_ns(&self) -> bool {
        for obj in &self.resources.object {
            if let Some(kind) = &obj.kind
                && let ObjectKind::Mesh(mesh) = kind
                && mesh.beamlattice.is_some()
            {
                return true;
            }
        }

        false
    }

    fn uses_triangleset_ns(&self) -> bool {
        for obj in &self.resources.object {
            if let Some(kind) = &obj.kind
                && let ObjectKind::Mesh(mesh) = kind
                && mesh.trianglesets.is_some()
            {
                return true;
            }
        }

        false
    }

    fn uses_boolean_ns(&self) -> bool {
        for obj in &self.resources.object {
            if let Some(kind) = &obj.kind
                && let ObjectKind::BooleanShape(_) = kind
            {
                return true;
            }
        }

        false
    }
}

#[cfg(feature = "write")]
#[cfg(test)]
mod write_tests {
    use instant_xml::{ToXml, to_string};
    use pretty_assertions::assert_eq;

    use crate::{
        core::{
            OptionalResourceId, OptionalResourceIndex,
            build::{Build, Item},
            mesh::{Mesh, Triangles, Vertices},
            metadata::Metadata,
            object::{Object, ObjectKind, ObjectType},
            resources::Resources,
        },
        threemf_namespaces::{
            BEAM_LATTICE_NS, BEAM_LATTICE_PREFIX, BOOLEAN_NS, BOOLEAN_PREFIX, CORE_NS,
            CORE_TRIANGLESET_NS, CORE_TRIANGLESET_PREFIX, PROD_NS, PROD_PREFIX, ThreemfNamespace,
        },
    };

    use super::{Model, Unit};

    #[test]
    pub fn toxml_simple_model_test() {
        let xml_string = format!(
            r#"<model xmlns="{CORE_NS}" xmlns:{BEAM_LATTICE_PREFIX}="{BEAM_LATTICE_NS}" xmlns:{BOOLEAN_PREFIX}="{BOOLEAN_NS}" xmlns:{PROD_PREFIX}="{PROD_NS}" xmlns:{CORE_TRIANGLESET_PREFIX}="{CORE_TRIANGLESET_NS}" unit="millimeter"><metadata name="Trial Metadata" /><resources><object id="346" type="model" name="test part"></object></resources><build><item objectid="346" /></build></model>"#,
        );
        let model = Model {
            // xmlns: None,
            unit: Some(Unit::Millimeter),
            requiredextensions: None,
            recommendedextensions: None,
            metadata: vec![Metadata {
                name: "Trial Metadata".to_owned(),
                preserve: None,
                value: None,
            }],
            resources: Resources {
                basematerials: vec![],
                object: vec![Object {
                    id: 346,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: Some("test part".to_owned()),
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    uuid: None,
                    kind: None,
                }],
            },
            build: Build {
                uuid: None,
                item: vec![Item {
                    objectid: 346,
                    transform: None,
                    partnumber: None,
                    path: None,
                    uuid: None,
                }],
            },
        };
        let model_string = to_string(&model).unwrap();

        assert_eq!(model_string, xml_string);
    }

    #[derive(Debug, ToXml, PartialEq, Eq)]
    struct UnitsType {
        unit: Vec<Unit>,
    }

    #[test]
    pub fn toxml_units_test() {
        let xml_string = "<UnitsType><unit>micron</unit><unit>millimeter</unit><unit>centimeter</unit><unit>inch</unit><unit>foot</unit><unit>meter</unit></UnitsType>";
        let unitsvector = UnitsType {
            unit: vec![
                Unit::Micron,
                Unit::Millimeter,
                Unit::Centimeter,
                Unit::Inch,
                Unit::Foot,
                Unit::Meter,
            ],
        };
        let unitsvector_string = to_string(&unitsvector).unwrap();

        assert_eq!(unitsvector_string, xml_string);
    }

    #[test]
    fn test_used_namespaces_simple_model() {
        let model = Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: None,
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: None,
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    uuid: None,
                    kind: Some(ObjectKind::Mesh(Mesh {
                        vertices: Vertices { vertex: vec![] },
                        triangles: Triangles { triangle: vec![] },
                        trianglesets: None,
                        beamlattice: None,
                    })),
                }],
                basematerials: vec![],
            },
            build: Build {
                uuid: None,
                item: vec![Item {
                    objectid: 1,
                    transform: None,
                    partnumber: None,
                    path: None,
                    uuid: None,
                }],
            },
        };

        let namespaces = model.used_namespaces();
        assert_eq!(namespaces, vec![ThreemfNamespace::Core]);
    }

    #[test]
    fn test_used_namespaces_with_prod() {
        let model = Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: None,
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: None,
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    uuid: Some("test-uuid".to_string()),
                    kind: Some(ObjectKind::Mesh(Mesh {
                        vertices: Vertices { vertex: vec![] },
                        triangles: Triangles { triangle: vec![] },
                        trianglesets: None,
                        beamlattice: None,
                    })),
                }],
                basematerials: vec![],
            },
            build: Build {
                uuid: None,
                item: vec![Item {
                    objectid: 1,
                    transform: None,
                    partnumber: None,
                    path: None,
                    uuid: None,
                }],
            },
        };

        let namespaces = model.used_namespaces();
        assert_eq!(
            namespaces,
            vec![ThreemfNamespace::Core, ThreemfNamespace::Prod]
        );
    }

    #[test]
    fn test_used_namespaces_with_beamlattice() {
        use crate::core::beamlattice::BeamLattice;

        let model = Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: None,
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: None,
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    uuid: None,
                    kind: Some(ObjectKind::Mesh(Mesh {
                        vertices: Vertices { vertex: vec![] },
                        triangles: Triangles { triangle: vec![] },
                        trianglesets: None,
                        beamlattice: Some(BeamLattice {
                            minlength: 0.1,
                            radius: 0.05,
                            ballmode: None,
                            ballradius: None,
                            clippingmode: None,
                            clippingmesh: OptionalResourceId::none(),
                            representationmesh: OptionalResourceId::none(),
                            pid: OptionalResourceId::none(),
                            pindex: OptionalResourceIndex::none(),
                            cap: None,
                            beams: crate::core::beamlattice::Beams { beam: vec![] },
                            balls: None,
                            beamsets: None,
                        }),
                    })),
                    // mesh: Some(Mesh {
                    //     vertices: Vertices { vertex: vec![] },
                    //     triangles: Triangles { triangle: vec![] },
                    //     trianglesets: None,
                    //     beamlattice: Some(BeamLattice {
                    //         minlength: 0.1,
                    //         radius: 0.05,
                    //         ballmode: None,
                    //         ballradius: None,
                    //         clippingmode: None,
                    //         clippingmesh: OptionalResourceId::none(),
                    //         representationmesh: OptionalResourceId::none(),
                    //         pid: OptionalResourceId::none(),
                    //         pindex: OptionalResourceIndex::none(),
                    //         cap: None,
                    //         beams: crate::core::beamlattice::Beams { beam: vec![] },
                    //         balls: None,
                    //         beamsets: None,
                    //     }),
                    // }),
                }],
                basematerials: vec![],
            },
            build: Build {
                uuid: None,
                item: vec![Item {
                    objectid: 1,
                    transform: None,
                    partnumber: None,
                    path: None,
                    uuid: None,
                }],
            },
        };

        let namespaces = model.used_namespaces();
        assert_eq!(
            namespaces,
            vec![ThreemfNamespace::Core, ThreemfNamespace::BeamLattice]
        );
    }

    #[test]
    fn test_used_namespaces_with_trianglesets() {
        use crate::core::triangle_set::TriangleSets;

        let model = Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: None,
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: None,
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    uuid: None,
                    kind: Some(ObjectKind::Mesh(Mesh {
                        vertices: Vertices { vertex: vec![] },
                        triangles: Triangles { triangle: vec![] },
                        trianglesets: Some(TriangleSets {
                            trianglesets: vec![],
                        }),
                        beamlattice: None,
                    })),
                    // mesh: Some(Mesh {
                    //     vertices: Vertices { vertex: vec![] },
                    //     triangles: Triangles { triangle: vec![] },
                    //     trianglesets: Some(TriangleSets {
                    //         trianglesets: vec![],
                    //     }),
                    //     beamlattice: None,
                    // }),
                }],
                basematerials: vec![],
            },
            build: Build {
                uuid: None,
                item: vec![Item {
                    objectid: 1,
                    transform: None,
                    partnumber: None,
                    path: None,
                    uuid: None,
                }],
            },
        };

        let namespaces = model.used_namespaces();
        assert_eq!(
            namespaces,
            vec![ThreemfNamespace::Core, ThreemfNamespace::CoreTriangleSet]
        );
    }

    #[test]
    fn test_used_namespaces_multiple_extensions() {
        use crate::core::{beamlattice::BeamLattice, triangle_set::TriangleSets};

        let model = Model {
            unit: Some(Unit::Millimeter),
            requiredextensions: None,
            recommendedextensions: None,
            metadata: vec![],
            resources: Resources {
                object: vec![Object {
                    id: 1,
                    objecttype: Some(ObjectType::Model),
                    thumbnail: None,
                    partnumber: None,
                    name: None,
                    pid: OptionalResourceId::none(),
                    pindex: OptionalResourceIndex::none(),
                    uuid: Some("test-uuid".to_string()),
                    kind: Some(ObjectKind::Mesh(Mesh {
                        vertices: Vertices { vertex: vec![] },
                        triangles: Triangles { triangle: vec![] },
                        trianglesets: Some(TriangleSets {
                            trianglesets: vec![],
                        }),
                        beamlattice: Some(BeamLattice {
                            minlength: 0.1,
                            radius: 0.05,
                            ballmode: None,
                            ballradius: None,
                            clippingmode: None,
                            clippingmesh: OptionalResourceId::none(),
                            representationmesh: OptionalResourceId::none(),
                            pid: OptionalResourceId::none(),
                            pindex: OptionalResourceIndex::none(),
                            cap: None,
                            beams: crate::core::beamlattice::Beams { beam: vec![] },
                            balls: None,
                            beamsets: None,
                        }),
                    })),
                    // mesh: Some(Mesh {
                    //     vertices: Vertices { vertex: vec![] },
                    //     triangles: Triangles { triangle: vec![] },
                    //     trianglesets: Some(TriangleSets {
                    //         trianglesets: vec![],
                    //     }),
                    //     beamlattice: Some(BeamLattice {
                    //         minlength: 0.1,
                    //         radius: 0.05,
                    //         ballmode: None,
                    //         ballradius: None,
                    //         clippingmode: None,
                    //         clippingmesh: OptionalResourceId::none(),
                    //         representationmesh: OptionalResourceId::none(),
                    //         pid: OptionalResourceId::none(),
                    //         pindex: OptionalResourceIndex::none(),
                    //         cap: None,
                    //         beams: crate::core::beamlattice::Beams { beam: vec![] },
                    //         balls: None,
                    //         beamsets: None,
                    //     }),
                    // }),
                }],
                basematerials: vec![],
            },
            build: Build {
                uuid: None,
                item: vec![Item {
                    objectid: 1,
                    transform: None,
                    partnumber: None,
                    path: None,
                    uuid: None,
                }],
            },
        };

        let namespaces = model.used_namespaces();
        assert_eq!(
            namespaces,
            vec![
                ThreemfNamespace::Core,
                ThreemfNamespace::Prod,
                ThreemfNamespace::BeamLattice,
                ThreemfNamespace::CoreTriangleSet
            ]
        );
    }
}

#[cfg(feature = "memory-optimized-read")]
#[cfg(test)]
mod memory_optimized_read_tests {
    use instant_xml::FromXml;
    use instant_xml::from_str;
    use pretty_assertions::assert_eq;

    use crate::core::OptionalResourceId;
    use crate::core::OptionalResourceIndex;
    use crate::{
        core::{
            build::{Build, Item},
            component::{Component, Components},
            metadata::Metadata,
            object::{Object, ObjectKind, ObjectType},
            resources::Resources,
        },
        threemf_namespaces::{CORE_NS, PROD_NS},
    };

    use super::{Model, Unit};

    #[test]
    pub fn fromxml_simple_model_test() {
        let xml_string = format!(
            r#"<model xmlns="{}"><metadata name="Trial Metadata" /><resources><object id="346" type="model" name="test part"></object></resources><build><item objectid="346" /></build></model>"#,
            CORE_NS
        );

        let model = from_str::<Model>(&xml_string).unwrap();

        assert_eq!(
            model,
            Model {
                unit: None, //ToDo: Set the default value when unit is not supplied.
                requiredextensions: None,
                recommendedextensions: None,
                metadata: vec![Metadata {
                    name: "Trial Metadata".to_owned(),
                    preserve: None,
                    value: None,
                }],
                resources: Resources {
                    basematerials: vec![],
                    object: vec![Object {
                        id: 346,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("test part".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: None,
                    }],
                },
                build: Build {
                    uuid: None,
                    item: vec![Item {
                        objectid: 346,
                        transform: None,
                        partnumber: None,
                        path: None,
                        uuid: None,
                    }],
                },
            }
        );
    }

    #[test]
    pub fn fromxml_production_model_test() {
        const CUSTOM_PROD_PREFIX: &str = "custom";
        let xml_string = format!(
            r#"<model xmlns="{}" xmlns:{}="{}" xml:lang="en-us" unit="millimeter"><metadata name="Trial Metadata" /><resources><object id="346" type="model" name="test part" {}:UUID="someObjectUUID"><components><component objectid="1" {}:path="//somePath//Component" {}:UUID="someComponentUUID" /></components></object></resources><build {}:UUID="someBuildUUID"><item objectid="346" {}:UUID="someItemUUID"/></build></model>"#,
            CORE_NS,
            CUSTOM_PROD_PREFIX,
            PROD_NS,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
        );
        let model = from_str::<Model>(&xml_string).unwrap();

        assert_eq!(
            model,
            Model {
                // xmlns: None,
                unit: Some(Unit::Millimeter),
                requiredextensions: None,
                recommendedextensions: None,
                metadata: vec![Metadata {
                    name: "Trial Metadata".to_owned(),
                    preserve: None,
                    value: None,
                }],
                resources: Resources {
                    basematerials: vec![],
                    object: vec![Object {
                        id: 346,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("test part".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: Some("someObjectUUID".to_owned()),
                        kind: Some(ObjectKind::Components(Components {
                            component: vec![Component {
                                objectid: 1,
                                transform: None,
                                path: Some("//somePath//Component".to_owned()),
                                uuid: Some("someComponentUUID".to_owned()),
                            }]
                        })),
                        // mesh: none,
                        // components: some(components {
                        //     component: vec![component {
                        //         objectid: 1,
                        //         transform: none,
                        //         path: some("//somepath//component".to_owned()),
                        //         uuid: some("somecomponentuuid".to_owned()),
                        //     }]
                        // }),
                    }],
                },
                build: Build {
                    uuid: Some("someBuildUUID".to_owned()),
                    item: vec![Item {
                        objectid: 346,
                        transform: None,
                        partnumber: None,
                        path: None,
                        uuid: Some("someItemUUID".to_owned()),
                    }],
                },
            }
        );
    }

    #[derive(FromXml, Debug, PartialEq, Eq)]
    struct UnitsType {
        unit: Vec<Unit>,
        #[xml(rename = "attr", attribute)]
        attribute: Option<Unit>,
    }

    #[test]
    pub fn fromxml_units_test() {
        let xml_string = r#"<UnitsType attr="inch"><unit>micron</unit><unit>millimeter</unit><unit>centimeter</unit><unit>inch</unit><unit>foot</unit><unit>meter</unit></UnitsType>"#;
        let unitsvector = from_str::<UnitsType>(xml_string).unwrap();

        assert_eq!(
            unitsvector,
            UnitsType {
                attribute: Some(Unit::Inch),
                unit: vec![
                    Unit::Micron,
                    Unit::Millimeter,
                    Unit::Centimeter,
                    Unit::Inch,
                    Unit::Foot,
                    Unit::Meter,
                ],
            }
        );
    }
}

#[cfg(feature = "speed-optimized-read")]
#[cfg(test)]
mod speed_optimized_read_tests {
    use pretty_assertions::assert_eq;
    use serde::Deserialize;
    use serde_roxmltree::from_str;

    use crate::{
        core::{
            OptionalResourceId, OptionalResourceIndex,
            build::{Build, Item},
            component::{Component, Components},
            metadata::Metadata,
            object::{Object, ObjectKind, ObjectType},
            resources::Resources,
        },
        threemf_namespaces::{CORE_NS, PROD_NS},
    };

    use super::{Model, Unit};

    #[test]
    pub fn fromxml_simple_model_test() {
        let xml_string = format!(
            r#"<model xmlns="{}"><metadata name="Trial Metadata" /><resources><object id="346" type="model" name="test part"></object></resources><build><item objectid="346" /></build></model>"#,
            CORE_NS
        );

        let model = from_str::<Model>(&xml_string).unwrap();

        assert_eq!(
            model,
            Model {
                // xmlns: None,
                unit: None, //ToDo: Set the default value when unit is not supplied.
                requiredextensions: None,
                recommendedextensions: None,
                metadata: vec![Metadata {
                    name: "Trial Metadata".to_owned(),
                    preserve: None,
                    value: Some("".to_string()), //ToDo: Import output for empty value
                }],
                resources: Resources {
                    basematerials: vec![],
                    object: vec![Object {
                        id: 346,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("test part".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: None,
                        kind: None,
                    }],
                },
                build: Build {
                    uuid: None,
                    item: vec![Item {
                        objectid: 346,
                        transform: None,
                        partnumber: None,
                        path: None,
                        uuid: None,
                    }],
                },
            }
        );
    }

    #[test]
    pub fn fromxml_production_model_test() {
        const CUSTOM_PROD_PREFIX: &str = "custom";
        let xml_string = format!(
            r#"<model xmlns="{}" xmlns:{}="{}" xml:lang="en-us" unit="millimeter"><metadata name="Trial Metadata" /><resources><object id="346" type="model" name="test part" {}:UUID="someObjectUUID"><components><component objectid="1" {}:path="//somePath//Component" {}:UUID="someComponentUUID" /></components></object></resources><build {}:UUID="someBuildUUID"><item objectid="346" {}:UUID="someItemUUID"/></build></model>"#,
            CORE_NS,
            CUSTOM_PROD_PREFIX,
            PROD_NS,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
            CUSTOM_PROD_PREFIX,
        );
        let model = from_str::<Model>(&xml_string).unwrap();

        assert_eq!(
            model,
            Model {
                // xmlns: None,
                unit: Some(Unit::Millimeter),
                requiredextensions: None,
                recommendedextensions: None,
                metadata: vec![Metadata {
                    name: "Trial Metadata".to_owned(),
                    preserve: None,
                    value: Some("".to_string()), //ToDo: Improve output for empty value
                }],
                resources: Resources {
                    basematerials: vec![],
                    object: vec![Object {
                        id: 346,
                        objecttype: Some(ObjectType::Model),
                        thumbnail: None,
                        partnumber: None,
                        name: Some("test part".to_owned()),
                        pid: OptionalResourceId::none(),
                        pindex: OptionalResourceIndex::none(),
                        uuid: Some("someObjectUUID".to_owned()),
                        kind: Some(ObjectKind::Components(Components {
                            component: vec![Component {
                                objectid: 1,
                                transform: None,
                                path: Some("//somePath//Component".to_owned()),
                                uuid: Some("someComponentUUID".to_owned()),
                            }]
                        })),
                        // components: Some(Components {
                        //     component: vec![Component {
                        //         objectid: 1,
                        //         transform: None,
                        //         path: Some("//somePath//Component".to_owned()),
                        //         uuid: Some("someComponentUUID".to_owned()),
                        //     }]
                        // }),
                    }],
                },
                build: Build {
                    uuid: Some("someBuildUUID".to_owned()),
                    item: vec![Item {
                        objectid: 346,
                        transform: None,
                        partnumber: None,
                        path: None,
                        uuid: Some("someItemUUID".to_owned()),
                    }],
                },
            }
        );
    }

    #[derive(Deserialize, Debug, PartialEq, Eq)]
    struct UnitsType {
        // #[serde(rename = "unit")]
        unit: Vec<Unit>,
        #[serde(rename = "attr")]
        attribute: Option<Unit>,
    }

    #[test]
    pub fn fromxml_units_test() {
        let xml_string = r#"<UnitsType attr="Inch"><unit>micron</unit><unit>millimeter</unit><unit>centimeter</unit><unit>inch</unit><unit>foot</unit><unit>meter</unit></UnitsType>"#;
        let unitsvector = from_str::<UnitsType>(xml_string).unwrap();

        assert_eq!(
            unitsvector,
            UnitsType {
                attribute: Some(Unit::Inch),
                unit: vec![
                    Unit::Micron,
                    Unit::Millimeter,
                    Unit::Centimeter,
                    Unit::Inch,
                    Unit::Foot,
                    Unit::Meter,
                ],
            }
        );
    }
}
