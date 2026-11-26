//! Foreign content on list entries (`<vertex>`, `<beam>`, `<m:tex2coord>`, ...), which is kept by
//! the list element in an `ItemAttributeExtensions` / `ItemAnyExtensions` table, with the `any`
//! feature.

#[cfg(all(feature = "any", feature = "read", feature = "write"))]
mod tests {
    use pretty_assertions::assert_eq;

    use threemf2::model::domain::{
        any::{AnyAttributes, AnyElement},
        any_extensions::{ItemAnyExtensions, ItemAttributeExtensions},
        beamlattice, displacement, material, mesh, slice, triangle_set,
    };
    use threemf2::threemf_namespaces::{
        BEAM_LATTICE_BALLS_NS, BEAM_LATTICE_NS, CORE_NS, CORE_TRIANGLESET_NS, DISPLACEMENT_NS,
        MATERIAL_NS, SLICE_NS,
    };

    const ACME_NS: &str = "http://acme.example/3mf/2026";

    fn acme(value: &str) -> AnyAttributes {
        let mut attributes = AnyAttributes::new();
        attributes.set(ACME_NS, "w", value).unwrap();
        attributes
    }

    /// The two per-entry table types, as far as [`check`] needs them.
    trait Table {
        fn attributes_of(&self, index: usize) -> Option<&AnyAttributes>;
        fn count(&self) -> usize;
        fn wipe(&mut self);
    }

    impl Table for ItemAttributeExtensions {
        fn attributes_of(&self, index: usize) -> Option<&AnyAttributes> {
            self.get(index)
        }
        fn count(&self) -> usize {
            self.len()
        }
        fn wipe(&mut self) {
            self.clear()
        }
    }

    impl Table for ItemAnyExtensions {
        fn attributes_of(&self, index: usize) -> Option<&AnyAttributes> {
            self.get(index).map(|content| content.attributes())
        }
        fn count(&self) -> usize {
            self.len()
        }
        fn wipe(&mut self) {
            self.clear()
        }
    }

    /// Reads `xml`, whose first `tag` entry has `acme:w="2"`, and checks that:
    /// - the attribute is stored in the table returned by `table` under index 0,
    /// - writing produces exactly the output without the table, plus the namespace declaration on
    ///   the list element and the attribute on that entry,
    /// - reading the output gives the same value.
    fn check<T, Tb: Table>(xml: &str, tag: &str, table: fn(&mut T) -> &mut Tb)
    where
        T: for<'xml> instant_xml::FromXml<'xml>
            + instant_xml::ToXml
            + Clone
            + PartialEq
            + std::fmt::Debug,
    {
        let mut value = instant_xml::from_str::<T>(xml).expect("read");
        assert_eq!(
            table(&mut value).attributes_of(0),
            Some(&acme("2")),
            "{tag}: table"
        );
        assert_eq!(table(&mut value).count(), 1, "{tag}: only the first entry");

        let written = instant_xml::to_string(&value).expect("write");

        let mut plain = value.clone();
        table(&mut plain).wipe();
        let mut expected = instant_xml::to_string(&plain).expect("write without table");
        let start_end = expected.find('>').unwrap();
        let start_end = match expected[..start_end].ends_with('/') {
            true => start_end - 1,
            false => start_end,
        };
        expected.insert_str(start_end, &format!(r#" xmlns:acme="{ACME_NS}""#));
        let entry = [format!("<{tag} "), format!(":{tag} ")]
            .iter()
            .filter_map(|needle| expected.find(needle.as_str()))
            .min()
            .unwrap_or_else(|| panic!("{tag}: no entry in {expected}"));
        let entry_end = entry + expected[entry..].find(" />").unwrap();
        expected.insert_str(entry_end, r#" acme:w="2""#);
        assert_eq!(written, expected, "{tag}: written XML");

        let reread = instant_xml::from_str::<T>(&written).expect("read back");
        assert_eq!(reread, value, "{tag}: round trip");
    }

    #[test]
    fn entry_attributes_roundtrip_in_every_list() {
        let a = format!(r#"xmlns:acme="{ACME_NS}""#);

        check::<mesh::Vertices, _>(
            &format!(
                r#"<vertices xmlns="{CORE_NS}" {a}><vertex x="0" y="0" z="0" acme:w="2" /><vertex x="1" y="0" z="0" /></vertices>"#
            ),
            "vertex",
            |v| &mut v.vertex_any_extensions,
        );
        check::<mesh::Triangles, _>(
            &format!(
                r#"<triangles xmlns="{CORE_NS}" {a}><triangle v1="0" v2="1" v3="2" p1="1" pid="3" acme:w="2" /></triangles>"#
            ),
            "triangle",
            |v| &mut v.triangle_any_extensions,
        );
        check::<displacement::Vertices, _>(
            &format!(
                r#"<vertices xmlns="{DISPLACEMENT_NS}" {a}><vertex x="0" y="0" z="0" acme:w="2" /></vertices>"#
            ),
            "vertex",
            |v| &mut v.vertex_any_extensions,
        );
        check::<displacement::Triangles, _>(
            &format!(
                r#"<triangles xmlns="{DISPLACEMENT_NS}" {a} did="4"><triangle v1="0" v2="1" v3="2" d1="0" acme:w="2" /></triangles>"#
            ),
            "triangle",
            |v| &mut v.triangle_any_extensions,
        );
        check::<displacement::NormVectorGroup, _>(
            &format!(
                r#"<normvectorgroup xmlns="{DISPLACEMENT_NS}" {a} id="1"><normvector x="0" y="0" z="1" acme:w="2" /></normvectorgroup>"#
            ),
            "normvector",
            |v| &mut v.normvector_any_extensions,
        );
        check::<displacement::Disp2DGroup, _>(
            &format!(
                r#"<disp2dgroup xmlns="{DISPLACEMENT_NS}" {a} id="2" dispid="1" nid="1" height="0.5"><disp2dcoord u="0" v="1" n="0" f="0.25" acme:w="2" /><disp2dcoord u="1" v="1" n="0" /></disp2dgroup>"#
            ),
            "disp2dcoord",
            |v| &mut v.disp2dcoord_any_extensions,
        );
        check::<material::Texture2DGroup, _>(
            &format!(
                r#"<texture2dgroup xmlns="{MATERIAL_NS}" {a} id="3" texid="1"><tex2coord u="0.5" v="0.25" acme:w="2" /></texture2dgroup>"#
            ),
            "tex2coord",
            |v| &mut v.tex2coord_any_extensions,
        );
        check::<material::MultiProperties, _>(
            &format!(
                r#"<multiproperties xmlns="{MATERIAL_NS}" {a} id="4" pids="1 2"><multi pindices="0 1" acme:w="2" /></multiproperties>"#
            ),
            "multi",
            |v| &mut v.multi_any_extensions,
        );
        check::<beamlattice::Beams, _>(
            &format!(
                r#"<beams xmlns="{BEAM_LATTICE_NS}" {a}><beam v1="0" v2="1" r1="1.5" cap1="sphere" acme:w="2" /></beams>"#
            ),
            "beam",
            |v| &mut v.beam_any_extensions,
        );
        check::<beamlattice::Balls, _>(
            &format!(
                r#"<balls xmlns="{BEAM_LATTICE_BALLS_NS}" {a}><ball vindex="0" r="2" acme:w="2" /></balls>"#
            ),
            "ball",
            |v| &mut v.ball_any_extensions,
        );
        check::<beamlattice::BeamSet, _>(
            &format!(
                r#"<beamset xmlns="{BEAM_LATTICE_NS}" xmlns:b2="{BEAM_LATTICE_BALLS_NS}" {a} name="s"><ref index="0" acme:w="2" /><b2:ballref index="0" /></beamset>"#
            ),
            "ref",
            |v| &mut v.refs_any_extensions,
        );
        check::<beamlattice::BeamSet, _>(
            &format!(
                r#"<beamset xmlns="{BEAM_LATTICE_NS}" xmlns:b2="{BEAM_LATTICE_BALLS_NS}" {a} name="s"><ref index="0" /><b2:ballref index="0" acme:w="2" /></beamset>"#
            ),
            "ballref",
            |v| &mut v.ballref_any_extensions,
        );
        check::<triangle_set::TriangleSet, _>(
            &format!(
                r#"<triangleset xmlns="{CORE_TRIANGLESET_NS}" {a} name="A" identifier="a"><ref index="2" acme:w="2" /><refrange startindex="3" endindex="5" /></triangleset>"#
            ),
            "ref",
            |v| &mut v.triangle_ref_any_extensions,
        );
        check::<triangle_set::TriangleSet, _>(
            &format!(
                r#"<triangleset xmlns="{CORE_TRIANGLESET_NS}" {a} name="A" identifier="a"><ref index="2" /><refrange startindex="3" endindex="5" acme:w="2" /></triangleset>"#
            ),
            "refrange",
            |v| &mut v.triangle_refrange_any_extensions,
        );
        check::<slice::Vertices, _>(
            &format!(
                r#"<vertices xmlns="{SLICE_NS}" {a}><vertex x="0" y="0" acme:w="2" /><vertex x="1" y="0" /></vertices>"#
            ),
            "vertex",
            |v| &mut v.vertex_any_extensions,
        );
        check::<slice::Polygon, _>(
            &format!(
                r#"<polygon xmlns="{SLICE_NS}" {a} startv="0"><segment v2="1" p1="0" acme:w="2" /></polygon>"#
            ),
            "segment",
            |v| &mut v.segment_any_extensions,
        );
    }

    #[test]
    fn entry_child_elements_roundtrip_on_beams_balls_and_refs() {
        let a = format!(r#"xmlns:acme="{ACME_NS}""#);

        // Elements only, attributes and elements, and a plain entry after them.
        let lattice: beamlattice::BeamLattice = roundtrip(&format!(
            r#"<beamlattice xmlns="{BEAM_LATTICE_NS}" xmlns:b2="{BEAM_LATTICE_BALLS_NS}" {a} minlength="0.0001" radius="1"><beams><beam v1="0" v2="1"><acme:strain v="0.1">x<acme:part/></acme:strain></beam><beam v1="1" v2="2" acme:w="2"><acme:note/></beam><beam v1="2" v2="0" /></beams><b2:balls><b2:ball vindex="0"><acme:note/></b2:ball></b2:balls><beamsets><beamset><ref index="0"><acme:note/></ref><b2:ballref index="0" acme:w="3"><acme:note/></b2:ballref></beamset></beamsets></beamlattice>"#
        ));
        let beams = &lattice.beams.beam_any_extensions;
        assert_eq!(beams.iter().map(|(i, _)| i).collect::<Vec<_>>(), [0, 1]);
        let strain = &beams.get(0).unwrap().elements()[0];
        assert_eq!(strain.name(), "strain");
        assert_eq!(strain.attribute("", "v"), Some("0.1"));
        assert_eq!(
            strain.child(ACME_NS, "part").map(AnyElement::name),
            Some("part")
        );
        assert!(beams.get(0).unwrap().attributes().is_empty());
        assert_eq!(
            beams.get(1).unwrap().attributes().get(ACME_NS, "w"),
            Some("2")
        );
        assert_eq!(beams.get(1).unwrap().elements()[0].name(), "note");
        let balls = &lattice.balls.as_ref().unwrap().ball_any_extensions;
        assert_eq!(balls.get(0).unwrap().elements().len(), 1);
        let set = &lattice.beamsets.as_ref().unwrap().beamset[0];
        assert_eq!(set.refs_any_extensions.get(0).unwrap().elements().len(), 1);
        let ballref = set.ballref_any_extensions.get(0).unwrap();
        assert_eq!(ballref.attributes().get(ACME_NS, "w"), Some("3"));
        assert_eq!(ballref.elements().len(), 1);

        let written = instant_xml::to_string(&lattice).unwrap();
        assert!(
            written.contains(r#"<beam v1="2" v2="0" />"#),
            "plain entries stay self-closing: {written}"
        );

        let set: triangle_set::TriangleSet = roundtrip(&format!(
            r#"<triangleset xmlns="{CORE_TRIANGLESET_NS}" {a} name="A" identifier="a"><ref index="2"><acme:note/></ref><refrange startindex="3" endindex="5"><acme:note n="1"/><acme:note n="2"/></refrange></triangleset>"#
        ));
        assert_eq!(
            set.triangle_ref_any_extensions
                .get(0)
                .unwrap()
                .elements()
                .len(),
            1
        );
        assert_eq!(
            set.triangle_refrange_any_extensions
                .get(0)
                .unwrap()
                .elements()
                .len(),
            2
        );
    }

    #[test]
    fn own_namespace_children_of_entries_are_skipped() {
        let beams: beamlattice::Beams = instant_xml::from_str(&format!(
            r#"<beams xmlns="{BEAM_LATTICE_NS}"><beam v1="0" v2="1"><unknown/></beam></beams>"#
        ))
        .unwrap();
        assert!(beams.beam_any_extensions.is_empty());
        assert_eq!(beams.beam.len(), 1);
    }

    #[test]
    fn writing_fails_for_a_non_foreign_entry_element() {
        let mut beams: beamlattice::Beams = instant_xml::from_str(&format!(
            r#"<beams xmlns="{BEAM_LATTICE_NS}"><beam v1="0" v2="1" /></beams>"#
        ))
        .unwrap();
        // `elements_mut` skips validation; writing catches the extension's own namespace.
        beams
            .beam_any_extensions
            .entry(0)
            .elements_mut()
            .push(AnyElement::new(BEAM_LATTICE_NS, "note").unwrap());
        assert!(instant_xml::to_string(&beams).is_err());
    }

    fn roundtrip<T>(xml: &str) -> T
    where
        T: for<'xml> instant_xml::FromXml<'xml> + instant_xml::ToXml + PartialEq + std::fmt::Debug,
    {
        let value = instant_xml::from_str::<T>(xml).expect("read");
        let written = instant_xml::to_string(&value).expect("write");
        let reread = instant_xml::from_str::<T>(&written).expect("read back");
        assert_eq!(reread, value, "round trip; written: {written}");
        value
    }

    #[test]
    fn entry_attributes_are_read_inside_a_whole_mesh() {
        let mesh: mesh::Mesh = instant_xml::from_str(&format!(
            r#"<mesh xmlns="{CORE_NS}" xmlns:acme="{ACME_NS}"><vertices><vertex x="0" y="0" z="0" /><vertex x="1" y="0" z="0" acme:w="1" /><vertex x="0" y="1" z="0" /></vertices><triangles><triangle v1="0" v2="1" v3="2" acme:w="3" acme:k="x" /></triangles></mesh>"#
        ))
        .expect("read");

        let vertices = &mesh.vertices.vertex_any_extensions;
        assert_eq!(vertices.iter().map(|(i, _)| i).collect::<Vec<_>>(), [1]);
        assert_eq!(vertices.get(1).unwrap().get(ACME_NS, "w"), Some("1"));
        let triangle = mesh.triangles.triangle_any_extensions.get(0).unwrap();
        assert_eq!(triangle.get(ACME_NS, "w"), Some("3"));
        assert_eq!(triangle.get(ACME_NS, "k"), Some("x"));
        // The schema attributes are unaffected.
        assert_eq!(mesh.vertices.vertex[1], mesh::Vertex::new(1.0, 0.0, 0.0));

        let written = instant_xml::to_string(&mesh).expect("write");
        let reread: mesh::Mesh = instant_xml::from_str(&written).expect("read back");
        assert_eq!(reread, mesh);
    }

    #[test]
    fn writing_fails_for_an_index_past_the_end_of_the_list() {
        let mut vertices = mesh::Vertices {
            vertex: vec![mesh::Vertex::new(0.0, 0.0, 0.0)],
            any_extensions: Default::default(),
            vertex_any_extensions: Default::default(),
        };
        vertices.vertex_any_extensions.insert(1, acme("2"));

        let error = instant_xml::to_string(&vertices).expect_err("stale index");
        assert!(error.to_string().contains("`vertex` 1"), "{error}");

        vertices.vertex.push(mesh::Vertex::new(1.0, 0.0, 0.0));
        let written = instant_xml::to_string(&vertices).expect("write");
        assert!(written.contains(r#"z="0" ns0:w="2" />"#), "{written}");
    }

    #[test]
    fn entries_with_empty_attribute_sets_are_written_plainly() {
        let mut vertices = mesh::Vertices {
            vertex: vec![mesh::Vertex::new(0.0, 0.0, 0.0)],
            any_extensions: Default::default(),
            vertex_any_extensions: Default::default(),
        };
        let plain = instant_xml::to_string(&vertices).expect("write");
        vertices.vertex_any_extensions.entry(0);

        assert_eq!(instant_xml::to_string(&vertices).expect("write"), plain);
    }
}

#[cfg(all(feature = "any", feature = "package-read", feature = "package-write"))]
mod package_tests {
    use pretty_assertions::assert_eq;

    use threemf2::model::{
        builder::{ModelBuilder, Unit},
        query::get_mesh_objects_from_model,
    };
    use threemf2::package::{ThreemfPackage, ThreemfPackageBuilder};

    use std::io::Cursor;

    const ACME_NS: &str = "http://acme.example/3mf/2026";

    #[test]
    fn builder_entry_attributes_survive_a_package_roundtrip() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        let object = builder
            .add_mesh_object(|obj| {
                obj.add_vertices(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
                    .add_triangle(&[0, 1, 2]);
                obj.vertex_any_extensions_mut()
                    .entry(2)
                    .set(ACME_NS, "weight", "0.5")?;
                obj.triangle_any_extensions_mut()
                    .entry(0)
                    .set(ACME_NS, "face", "top")?;
                obj.add_beam_lattice(|lattice| {
                    lattice.minlength(0.0001).radius(1.0).add_beam(0, 1);
                    let beam = lattice.beam_any_extensions_mut().entry(0);
                    beam.attributes_mut().set(ACME_NS, "load", "12").unwrap();
                    beam.push_element(
                        threemf2::model::domain::any::AnyElement::new(ACME_NS, "strain")
                            .unwrap()
                            .with_text("0.1"),
                    )
                    .unwrap();
                });
                Ok(())
            })
            .unwrap();
        builder.add_build(None).unwrap();
        builder.add_build_item(object).unwrap();

        let mut package_builder = ThreemfPackageBuilder::new();
        package_builder.set_root_model(builder.build().unwrap());
        let mut buf = Cursor::new(Vec::new());
        package_builder
            .build()
            .unwrap()
            .write(&mut buf)
            .expect("write package");
        buf.set_position(0);

        let package = ThreemfPackage::from_reader(&mut buf, false).expect("read package");
        let mesh = get_mesh_objects_from_model(&package.root).next().unwrap();
        let vertices = mesh.vertex_any_extensions();
        assert_eq!(vertices.len(), 1);
        assert_eq!(vertices.get(2).unwrap().get(ACME_NS, "weight"), Some("0.5"));
        assert_eq!(
            mesh.triangle_any_extensions()
                .get(0)
                .unwrap()
                .get(ACME_NS, "face"),
            Some("top")
        );
        let lattice = mesh.lattice().unwrap();
        let beam = lattice.beam_any_extensions().get(0).unwrap();
        assert_eq!(beam.attributes().get(ACME_NS, "load"), Some("12"));
        assert_eq!(beam.elements()[0].name(), "strain");
        assert_eq!(beam.elements()[0].text(), "0.1");
    }

    #[test]
    fn builder_rejects_entry_attributes_past_the_last_entry() {
        let mut builder = ModelBuilder::new(Unit::Millimeter, true);
        let object = builder
            .add_mesh_object(|obj| {
                obj.add_vertices(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
                    .add_triangle(&[0, 1, 2]);
                obj.vertex_any_extensions_mut()
                    .entry(3)
                    .set(ACME_NS, "weight", "0.5")?;
                Ok(())
            })
            .unwrap();
        builder.add_build(None).unwrap();
        builder.add_build_item(object).unwrap();

        let mut package_builder = ThreemfPackageBuilder::new();
        package_builder.set_root_model(builder.build().unwrap());
        let mut buf = Cursor::new(Vec::new());
        assert!(package_builder.build().unwrap().write(&mut buf).is_err());
    }
}
