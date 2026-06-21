use criterion::{Criterion, criterion_group, criterion_main};

use threemf2::model::domain::model::Model;

use std::path::PathBuf;

pub fn read_instant_xml(c: &mut Criterion) {
    let path = PathBuf::from("../threemf2-tests/tests/data/lfs/3dmodel.model")
        .canonicalize()
        .unwrap();
    let text = std::fs::read_to_string(path).unwrap();
    let mut c = c.benchmark_group("read_group");
    c.sample_size(10);
    c.measurement_time(std::time::Duration::from_secs(70));
    c.bench_function("read instant_xml function", |b| {
        b.iter(|| instant_xml::from_str::<Model>(&text).unwrap())
    });
}

pub fn read_roxmltree(c: &mut Criterion) {
    let path = PathBuf::from("../threemf2-tests/tests/data/lfs/3dmodel.model")
        .canonicalize()
        .unwrap();

    let text = std::fs::read_to_string(path).unwrap();
    let mut c = c.benchmark_group("read_group");
    c.sample_size(10);
    c.measurement_time(std::time::Duration::from_secs(40));
    c.bench_function("read roxmltree function", |b| {
        b.iter(|| serde_roxmltree::from_str::<Model>(&text).unwrap())
    });
}

criterion_group!(benches, read_instant_xml, read_roxmltree);
criterion_main!(benches);
