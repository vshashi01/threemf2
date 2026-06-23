# threemf2

Library for reading and writing 3MF (3D Manufacturing Format) packages with both eager and lazy loading support.

This repository uses a Cargo workspace with `threemf2` (main crate), `threemf2-benches` (benchmarks), and `threemf2-tests` (integration tests with large data). The `threemf2` crate provides a compact core model representation and I/O helpers for reading/writing 3MF packages with multiple loading strategies optimized for different use cases.

This crate is **100% safe Rust** — it explicitly forbids unsafe code (`#![forbid(unsafe_code)]`).

[![CI](https://github.com/vshashi01/threemf2/actions/workflows/CI.yml/badge.svg)](https://github.com/vshashi01/threemf2/actions/workflows/CI.yml) [![codecov](https://codecov.io/gh/vshashi01/threemf2/graph/badge.svg?token=O1EHCUZLT4)](https://codecov.io/gh/vshashi01/threemf2) [![docs.rs](https://docs.rs/threemf2/badge.svg)](https://docs.rs/threemf2)

## Supported 3MF Extensions and Maximum Supported Versions

| 3MF Specifications | Type      | Read<sup>1</sup> |  Write<sup>2</sup>   | Current supported version |
| ------------------ | --------- | :--------------: | :------------------: | ------------------------: |
| 3MF Core Spec      | Core      |       MUST       |         MUST         |                     1.3.0 |
| Triangle Set       | Extension |       MUST       |       OPTIONAL       |           Core Spec 1.3.0 |
| Production         | Extension |       MUST       |       OPTIONAL       |                     1.1.2 |
| Beam Lattice       | Extension |       MUST       |       OPTIONAL       |                     1.2.0 |
| Boolean Operations | Extension |       MUST       |       OPTIONAL       |                     1.1.1 |
| Slice              | Extension | MUST<sup>3</sup> | OPTIONAL<sup>3</sup> |                     1.0.2 |
| Materials          | Extension | MUST<sup>4</sup> | OPTIONAL<sup>4</sup> |                     1.2.1 |
| Displacement       | Extension | MUST<sup>5</sup> | OPTIONAL<sup>5</sup> |                     1.0.0 |

**Note: This library is still in active development, expect frequent API changes!!**

**Note<sub>1</sub>: Reading these data is currently MUST, which means if the data exists in the 3MF Model the library will automatically read it.**

**Note<sub>2</sub>: Writing these data is currently optional however they are available by default in the mapping of the Rust types and are not conditionally compiled out.**

**Note<sub>3</sub>: Slice extension data are read and written always as optional fields however the query and builder APIs are not stabilized hence expect future API changes.**

**Note<sub>4</sub>: Material extension data are read and written always as optional fields however the query and builder APIs are not stabilized hence expect future API changes. All display specific material properties defined in this extension are not supported currently.**

**Note<sub>5</sub>: Displacement extension data are read and written always as optional fields however the query and builder APIs are not stabilized hence expect future API changes.**

## Overview

threemf2 provides:

- **Core Data Structures**: Complete 3MF model representation ([`Model`](src/model/domain/model.rs), [`Object`](src/model/domain/object.rs), [`Mesh`](src/model/domain/mesh.rs), etc.)
- **Multiple Loading Strategies**:
  - [`package::ThreemfPackage`](src/package/threemf_package.rs) - Eager loading for complete data access
  - [`package::ThreemfPackageLazyReader`](src/package/threemf_package_lazy_reader.rs) - Lazy loading for memory-constrained environments
- **Flexible I/O**: Support for reading/writing 3MF packages with different performance characteristics
  - Easy reading of data through query APIs.
  - Easy creation of 3MF Model through builder APIs. See [`builder_example.rs`](examples/builder_example.rs) for the most basic starter guide.
- **Extension Support**: All 3MF extensions (Production, Beam Lattice, etc.) are always available — they are not behind feature flags because spec compliance is a core goal.
- **Custom Parts**: Support for known parts (thumbnails) and unknown parts (custom XML data)

## Quick Start

### Reading a 3MF file

```rust
use std::fs::File;
use threemf2::package::ThreemfPackage;

let file = File::open("model.3mf")?;
let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(file, true)?;

// Access the root model
let model = &package.root;

// Iterate over all mesh objects in the package
for mesh in threemf2::package::query::get_mesh_objects(&package) {
    println!("Object {} has {} vertices and {} triangles",
        mesh.view.id(),
        mesh.view.vertex_count(),
        mesh.view.triangle_count()
    );
}
```

### Building a 3MF file

```rust
use threemf2::model::builder::{ModelBuilder, ObjectType, Unit};
use threemf2::package::ThreemfPackageBuilder;

let mut builder = ModelBuilder::new(Unit::Millimeter, true);
builder.add_build(None).unwrap();

let obj_id = builder
    .add_mesh_object(|obj| {
        obj.object_type(ObjectType::Model);
        obj.add_vertices(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        obj.add_triangle(&[0, 1, 2]);
        Ok(())
    })
    .unwrap();

builder.add_build_item(obj_id).unwrap();

let model = builder.build().unwrap();

let mut package_builder = ThreemfPackageBuilder::new();
package_builder.set_root_model(model);
let package = package_builder.build().unwrap();

let mut bytes: Vec<u8> = vec![];
package.write(std::io::Cursor::new(&mut bytes)).unwrap();
```

## Performance Options

- **Memory-Optimized**: Lower memory usage, good for large files. This is the default. Parsing speed in the slowest path is 10-20% slower than speed-optimized configuration
- **Speed-Optimized (deprecated)**: Faster parsing however uses 1.5x to 3x more memory than memory-optimized
- **Lazy Loading**: Defers loading until accessed, best for inspection-only use cases

Key types and files:

- Core model types in [src/model/domain/](src/model/domain/) — `model`, `object`, `resources`, `mesh`, `transform`, etc.
- [`package::ThreemfPackage`](src/package/threemf_package.rs) — eager loading entry point
- [`package::ThreemfPackageLazyReader`](src/package/threemf_package_lazy_reader.rs) — lazy loading entry point

## Cargo Features

This crate uses optional Cargo features to control functionality. Enable only what you need.

### Core Serialization Features

- `write` — Enable writing 3MF data (adds `ToXml` derive to all 3MF types using `instant_xml`)
- `memory-optimized-read` — Enable memory-efficient reading (adds `FromXml` derive to all 3MF types using `instant_xml`)
- `speed-optimized-read` (deprecated) — Enable fast reading (adds `serde::Deserialize` derive to all 3MF types using `serde_roxmltree`)

### Package I/O Features

- `package-write` — Package writing with ZIP creation (requires `write`)
- `package-memory-optimized-read` — Package reading with memory optimization (requires `memory-optimized-read`)
- `io-speed-optimized-read` (deprecated) — Package reading with speed optimization (requires `speed-optimized-read`)
- `package-lazy-read` — Lazy loading functionality (requires `package-memory-optimized-read`)

### Utility Features

- `uuid` — Enable UUID validation for Production extension data (requires the `uuid` crate)

### Default Features

`package-write`, `package-memory-optimized-read`, `package-lazy-read`, `write`, `memory-optimized-read`

### Feature Combinations

```toml
# Basic reading (default features)
threemf2 = "0.4.0"

# Full I/O with lazy loading (default)
threemf2 = { version = "0.4.0", features = ["package-lazy-read"] }

# Memory-constrained environments
threemf2 = { version = "0.4.0", features = ["package-lazy-read"], default-features = false }

# High-performance reading (deprecated)
threemf2 = { version = "0.4.0", features = ["io-speed-optimized-read"] }

# With UUID validation
threemf2 = { version = "0.4.0", features = ["uuid"] }
```

## Examples

The [examples/](examples/) directory contains runnable examples for different use cases:

- **`write.rs`** — Create and write 3MF packages
- **`builder_example.rs`** — Using ModelBuilder for ergonomic model construction
- **`builder_beamlattice_example.rs`** — Building models with beam lattice extensions
- **`unpack.rs`** — Lazy loading with `ThreemfPackageLazyReader`
- **`package_memory_optimized_read.rs`** — Memory-efficient reading
- **`io_speed_optimized_read.rs`** — High-performance reading (deprecated)
- **`string_extraction.rs`** — Access raw XML content
- **`beamlattice_write.rs`** — Working with beam lattice extensions
- **`query_example.rs`** — Using the query API to inspect package contents
- **`boolean_write.rs`** — Creating boolean shape objects
- **`slice_write.rs`** — Creating slice stack data

Run examples with:

```bash
cargo run --example write --features package-write
cargo run --example unpack --features package-lazy-read
cargo run --example query_example --features package-memory-optimized-read,uuid
```

## Benchmarks

The benchmarks use Criterion.rs as the benchmark harness on stable Rust channel.
Benchmarks are located in the `threemf2-benches` crate. To run them, use `cargo bench --package threemf2-benches`. The files used for the benchmarks are on Git LFS, so ensure `git lfs` is enabled.

- **reader** — Benchmarks the different serialization options `instant-xml` and `serde-roxmltree` on an uncompressed 3MF model file.
- **threemf_reader** — Benchmarks the different reader methods on `ThreemfPackage`.
- **threemf_write** — Benchmarks the writer method on a full 3MF package.

## API Overview

### Core Data Structures

- `Model` — Root 3MF model with resources and build configuration
- `Object` — 3D objects (meshes, component assemblies, boolean shapes, or displacement meshes)
- `Mesh` — Triangle mesh geometry with vertices and triangles
- `Component` — Object references with transforms (for assembly objects)
- `Build` — Defines which objects should be printed

### Package I/O

- `ThreemfPackage` — Eager loading — loads all data upfront into memory
- `ThreemfPackageLazyReader` — Lazy loading — loads metadata first, defers model data until accessed

### Query APIs

- `model::query` — Model-level queries: iterate over objects, meshes, build items, slice stacks, materials, etc.
- `package::query` — Package-level queries: iterate over all objects/meshes/items across root and sub-models

### Builder APIs

- `model::builder` — Programmatically construct `Model` instances with automatic ID management and extension detection
- `package::builder` — Assemble multiple models into a complete 3MF package with proper ZIP structure, content types, and relationships

## Building & Testing

### Requirements

- Rust 1.89.0 or later (2024 edition)
- Cargo package manager

### Build Commands

```bash
# Build with all features
cargo build --all-features

# Check formatting and linting
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

### Testing

```bash
# Run all tests in the workspace (includes threemf2, threemf2-tests)
cargo test

# Run tests for specific crates
cargo test --package threemf2  # Core tests (CI-focused)
cargo test --package threemf2-tests  # Integration tests with large data (requires Git LFS)

# Install cargo-all-features and run tests for multiple feature combinations at once
cargo all-features test

# Run benchmarks
cargo bench --package threemf2-benches
```

## Contributing

Contributions are welcome.

- Open an issue to discuss major changes or report bugs.
- Fork the repo and create a feature branch.
- Add tests that exercise new behavior (tests may be feature-gated).
- Run tests locally with all possible features, preferably use `cargo all-features test`.
- Add or update an example.
- Add or update the documentation.
- See [`ARCHITECTURE.md`](ARCHITECTURE.md) for a detailed guide to the crate structure.
- Submit a pull request with a clear description and link to any related issue.

### AI-Assisted Contributions

We welcome contributions created with the assistance of AI tools. However, all contributors must:

- **Clearly disclose AI assistance** in your pull request description and commit messages.
- **Provide due diligence** by:
  - Testing the code thoroughly (run all tests with `cargo-all-features`, see above).
  - Reviewing the generated code for correctness and adherence to project conventions.
  - Understanding what the code does and why it works.
  - Ensuring the contribution follows the project's style and patterns.
- **Take responsibility** for the final code quality and functionality.

AI tools can be excellent for productivity, but human oversight and understanding remain essential for maintaining code quality.

By contributing you agree to license your contributions under MIT or Apache 2.0 licenses.

## License

This project and its source code are released under [MIT](/LICENSE-MIT) or [Apache 2.0](/LICENSE-APACHE) licenses.
