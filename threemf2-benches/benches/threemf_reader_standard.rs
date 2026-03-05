use criterion::{Criterion, criterion_group, criterion_main};

use threemf2::io::ThreemfPackage;

use std::path::PathBuf;

pub fn read_memory_optimized(c: &mut Criterion) {
    let path =
        PathBuf::from("../threemf2-tests/tests/data/third-party/lfs/mgx-iron_giant_single.3mf")
            .canonicalize()
            .unwrap();
    let mut c = c.benchmark_group("read_group");
    c.sample_size(10);
    c.measurement_time(std::time::Duration::from_secs(70));
    c.bench_function("Memory optimized read", |b| {
        b.iter(|| {
            let file = std::fs::File::open(path.clone()).unwrap();
            ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();
        });
    });
}

pub fn read_speed_optimized(c: &mut Criterion) {
    let path =
        PathBuf::from("../threemf2-tests/tests/data/third-party/lfs/mgx-iron_giant_single.3mf")
            .canonicalize()
            .unwrap();
    let mut c = c.benchmark_group("read_group");
    c.sample_size(10);
    c.measurement_time(std::time::Duration::from_secs(40));
    c.bench_function("read roxmltree function", |b| {
        b.iter(|| {
            let file = std::fs::File::open(path.clone()).unwrap();
            ThreemfPackage::from_reader_with_speed_optimized_deserializer(file, true).unwrap()
        })
    });
}

criterion_group!(benches, read_memory_optimized, read_speed_optimized);
criterion_main!(benches);
