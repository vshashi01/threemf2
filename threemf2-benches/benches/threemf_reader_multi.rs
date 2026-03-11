use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use lib3mf_core::archive::{ArchiveReader, ZipArchiver};
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
    let config: Config = serde_json::from_str(CONFIG_JSON).unwrap();
    let mut group = c.benchmark_group("memory_optimized");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(30));

    for file in &config.files {
        let path = PathBuf::from(format!("../threemf2-tests/tests/data/{}", file.path))
            .canonicalize()
            .unwrap();
        let name = get_short_name(&file.path);

        #[cfg(feature = "enable-alloc-check")]
        let _dhat = dhat::Profiler::builder()
            .file_name(format!("/target/dhat-memory-optimized-{name}.json"))
            .build();

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
    let config: Config = serde_json::from_str(CONFIG_JSON).unwrap();
    let mut group = c.benchmark_group("speed_optimized");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(30));

    for file in &config.files {
        let path = PathBuf::from(format!("../threemf2-tests/tests/data/{}", file.path))
            .canonicalize()
            .unwrap();
        let name = get_short_name(&file.path);

        #[cfg(feature = "enable-alloc-check")]
        let _dhat = dhat::Profiler::builder()
            .file_name(format!("/target/dhat-speed-optimized-{name}.json"))
            .build();
        group.bench_with_input(BenchmarkId::new("read", name), &path, |b, path| {
            b.iter(|| {
                let file = std::fs::File::open(path).unwrap();
                ThreemfPackage::from_reader_with_speed_optimized_deserializer(file, true).unwrap();
            });
        });
    }
    group.finish();
}

fn bench_lib3mf_rs(c: &mut Criterion) {
    let config: Config = serde_json::from_str(CONFIG_JSON).unwrap();
    let mut group = c.benchmark_group("lib3mf_rs");
    group.sample_size(10);
    group.measurement_time(std::time::Duration::from_secs(30));

    for file in &config.files {
        let path = PathBuf::from(format!("../threemf2-tests/tests/data/{}", file.path))
            .canonicalize()
            .unwrap();
        let name = get_short_name(&file.path);

        #[cfg(feature = "enable-alloc-check")]
        let _dhat = dhat::Profiler::builder()
            .file_name(format!("/target/dhat-speed-optimized-{name}.json"))
            .build();
        group.bench_with_input(BenchmarkId::new("read", name), &path, |b, path| {
            b.iter(|| {
                let file = std::fs::File::open(path).unwrap();
                let mut archiver = ZipArchiver::new(file).unwrap();

                let model_path = lib3mf_core::archive::find_model_path(&mut archiver).unwrap();
                let model_data = archiver.read_entry(&model_path).unwrap();

                lib3mf_core::parser::parse_model(std::io::Cursor::new(model_data)).unwrap();
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_memory_optimized,
    bench_speed_optimized,
    bench_lib3mf_rs
);
criterion_main!(benches);
