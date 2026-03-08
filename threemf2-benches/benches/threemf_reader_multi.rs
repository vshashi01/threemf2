use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use serde::Deserialize;
use std::path::PathBuf;
use threemf2::io::ThreemfPackage;

#[cfg(feature = "enable-alloc-check")]
#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

#[derive(Deserialize)]
struct Config {
    files: Vec<BenchFile>,
}

#[derive(Deserialize)]
struct BenchFile {
    path: String,
}

const CONFIG_JSON: &str = include_str!("../bench_config.json");

fn get_short_name(path: &str) -> String {
    PathBuf::from(path)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_string()
}

fn bench_memory_optimized(c: &mut Criterion) {
    #[cfg(feature = "enable-alloc-check")]
    let _dhat = dhat::Profiler::builder()
        .file_name("target/dhat-memory-optimized.json")
        .build();
    let config: Config = serde_json::from_str(CONFIG_JSON).unwrap();
    let mut group = c.benchmark_group("memory_optimized");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(30));

    for file in &config.files {
        let path = PathBuf::from(format!("../threemf2-tests/tests/data/{}", file.path))
            .canonicalize()
            .unwrap();
        let name = get_short_name(&file.path);

        group.bench_with_input(BenchmarkId::new("read", name), &path, |b, path| {
            b.iter(|| {
                let file = std::fs::File::open(path).unwrap();
                ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_speed_optimized(c: &mut Criterion) {
    #[cfg(feature = "enable-alloc-check")]
    let _dhat = dhat::Profiler::builder()
        .file_name("target/dhat-speed-optimized.json")
        .build();
    let config: Config = serde_json::from_str(CONFIG_JSON).unwrap();
    let mut group = c.benchmark_group("speed_optimized");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(30));

    for file in &config.files {
        let path = PathBuf::from(format!("../threemf2-tests/tests/data/{}", file.path))
            .canonicalize()
            .unwrap();
        let name = get_short_name(&file.path);

        group.bench_with_input(BenchmarkId::new("read", name), &path, |b, path| {
            b.iter(|| {
                let file = std::fs::File::open(path).unwrap();
                ThreemfPackage::from_reader_with_speed_optimized_deserializer(file, true).unwrap();
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_memory_optimized, bench_speed_optimized);
criterion_main!(benches);
