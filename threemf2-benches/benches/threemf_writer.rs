use criterion::{Criterion, criterion_group, criterion_main};

use threemf2::package::ThreemfPackage;

use std::{io::Cursor, path::PathBuf};

pub fn write_package(c: &mut Criterion) {
    let path =
        PathBuf::from("../threemf2-tests/tests/data/third-party/lfs/mgx-iron_giant_single.3mf")
            .canonicalize()
            .unwrap();
    let file = std::fs::File::open(path.clone()).unwrap();
    let package =
        ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();
    let mut c = c.benchmark_group("read_group");
    c.sample_size(10);
    c.measurement_time(std::time::Duration::from_secs(70));
    c.bench_function("Write package", |b| {
        b.iter(|| {
            let bytes = Vec::<u8>::new();
            let writer = Cursor::new(bytes);

            package.clone().write(writer).unwrap();
        });
    });
}

criterion_group!(benches, write_package);
criterion_main!(benches);
