# threemf2

Library for reading and writing 3MF (3D Manufacturing Format) packages with both eager and lazy loading support.

This repository uses a Cargo workspace with `threemf2` (main crate), `threemf2-benches` (benchmarks), and `threemf2-tests` (integration tests with large data). The `threemf2` crate provides a compact core model representation and I/O helpers for reading/writing 3MF packages with multiple loading strategies optimized for different use cases.

[![CI](https://github.com/vshashi01/threemf2/actions/workflows/CI.yml/badge.svg)](https://github.com/vshashi01/threemf2/actions/workflows/CI.yml) [![codecov](https://codecov.io/gh/vshashi01/threemf2/graph/badge.svg?token=O1EHCUZLT4)](https://codecov.io/gh/vshashi01/threemf2)

## Supported 3MF Extensions and Maximum Supported Versions

| 3MF Specifications | Type      | Read<sub>1</sub> | Write<sub>2</sub> | Current supported version |
| ------------------ | --------- | :--------------: | :---------------: | ------------------------: |
| 3MF Core Spec      | Core      |       MUST       |       MUST        |                     1.3.0 |
| Triangle Set       | Extension |       MUST       |     OPTIONAL      |           Core Spec 1.3.0 |
| Production         | Extension |       MUST       |     OPTIONAL      |                     1.1.2 |
| Beam Lattice       | Extension |       MUST       |     OPTIONAL      |                     1.2.0 |
| Boolean Operations | Extension |       MUST       |     OPTIONAL      |                     1.1.1 |

**Note: This library is still in active development, expect frequent API changes!!**

**Note<sub>1</sub>: Reading these data are currently MUST, which means if the data exists in the 3MF Model the library will automatically read it**

**Note<sub>2</sub>: Write these data are currently optional however they are available by default in the mapping of the Rust types and are not conditionally compiled.**

## Overview

threemf2 provides:

- **Core Data Structures**: Complete 3MF model representation ([`Model`](threemf2/src/core/model.rs), [`Object`](threemf2/src/core/object.rs), [`Mesh`](threemf2/src/core/mesh.rs), etc.)
- **Multiple Loading Strategies**:
  - [`ThreemfPackage`](threemf2/src/io/threemf_package.rs) - Eager loading for complete data access
  - [`ThreemfPackageLazyReader`](threemf2/src/io/threemf_package_lazy_reader.rs) - Lazy loading for memory-constrained environments
- **Flexible I/O**: Support for reading/writing 3MF packages with different performance characteristics
  - Easy reading of data through query APIs.
  - Easy creation of 3MF Model through builder APIs. See [`builder_example.rs`](threemf2/examples/builder_example.rs) for the most basic starter guide.
- **Extension Support**: All 3MF extensions (Production, Beam Lattice, etc.) are always available
- **Custom Parts**: Support for known parts (thumbnails) and unknown parts (custom XML data)

## Performance Options

Choose the right loading strategy for your use case:

- **Memory-Optimized**: Lower memory usage, good for large files. This is the default.
- **Speed-Optimized**: Faster parsing, higher memory usage
- **Lazy Loading**: Defers loading until accessed, best for inspection-only use cases

Key types and files:

- Core model types in [threemf2/src/core/](threemf2/src/core/) — `model`, `object`, `resources`, `mesh`, `transform`, etc.
- [`io::ThreemfPackage`](threemf2/src/io/threemf_package.rs) — eager loading entry point
- [`io::ThreemfPackageLazyReader`](threemf2/src/io/threemf_package_lazy_reader.rs) — lazy loading entry point

## Cargo Features

This crate uses optional Cargo features to control functionality. Enable only what you need.

### Core Serialization Features

- `write` — Enable writing 3MF data (adds `ToXml` derive to all 3MF types using `instant_xml`)
- `memory-optimized-read` — Enable memory-efficient reading (adds `FromXml` derive to all 3MF types using `instant_xml`)
- `speed-optimized-read` — Enable fast reading (adds `serde::Deserialize` derive to all 3MF types using `serde_roxmltree`)

### Package I/O Features

- `io-write` — Package writing with ZIP creation (requires `write`)
- `io-memory-optimized-read` — Package reading with memory optimization (requires `memory-optimized-read`)
- `io-speed-optimized-read` — Package reading with speed optimization (requires `speed-optimized-read`)
- `io-lazy-read` — Lazy loading functionality (requires `io-memory-optimized-read`)

### Default Features

`io-write`, `io-memory-optimized-read`, `io-lazy-read`, `write`, `memory-optimized-read`

### Feature Combinations

```toml
# Basic reading
threemf2 = "0.1"

# Full I/O with lazy loading (default)
threemf2 = { version = "0.1", features = ["io-lazy-read"] }

# Memory-constrained environments
threemf2 = { version = "0.1", features = ["io-lazy-read"], default-features = false }

# High-performance reading
threemf2 = { version = "0.1", features = ["io-speed-optimized-read"] }
```

## Examples

The [threemf2/examples/](threemf2/examples/) directory contains runnable examples for different use cases:

- **`write.rs`** - Create and write 3MF packages
- **`builder_example.rs`** - Using ModelBuilder for ergonomic model construction
- **`unpack.rs`** - Lazy loading with `ThreemfPackageLazyReader`
- **`io_memory_optimized_read.rs`** - Memory-efficient reading
- **`io_speed_optimized_read.rs`** - High-performance reading
- **`string_extraction.rs`** - Access raw XML content
- **`beamlattice_write.rs`** - Working with beam lattice extensions

Run examples with:

```bash
cargo run --example write --features io-write
cargo run --example unpack --features io-lazy-read
```

## Benchmarks

The benchmarks use Criterion.rs as the benchmark harness on stable Rust channel.
Benchmarks are located in the `threemf2-benches` crate. To run them, use `cargo bench --package threemf2-benches`. The files used for the benchmarks are on Git LFS, so ensure `git lfs` is enabled.

- **reader** - Benchmarks the different serialization options instant-xml and serde-roxmltree on an uncompressed 3MF model file.

- **threemf_reader** - Benchmarks the different reader methods on ThreemfPackage.

- **threemf_write** - Benchmarks the writer method on a full 3MF package.

## API Overview

### Core Data Structures

- `Model` - Root 3MF model with resources and build configuration
- `Object` - 3D objects (meshes or component assemblies)
- `Mesh` - Triangle mesh geometry with vertices and triangles
- `Component` - Object references with transforms

### Package I/O

- `ThreemfPackage` - Eager loading - loads all data upfront
- `ThreemfPackageLazyReader` - Lazy loading - loads metadata first, data on-demand

## Reading and Writing easily through `io` crate

- `query` module enables easy retrieval of data from the 3MF Package or 3MF model.
- `builder` enables easy creation of 3MF Model.

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

## License

This project and its source code are released under [MIT](/LICENSE-MIT) or [Apache 2.0](/LICENSE-APACHE) licenses.

## Contributing

Contributions are welcome.

- Open an issue to discuss major changes or report bugs.
- Fork the repo and create a feature branch.
- Add tests that exercise new behavior (tests may be feature-gated).
- Run tests locally with all possible features, preferably use `cargo all-features test`:
- Add or update an example
- Add or update the documentation.
- Submit a pull request with a clear description and link to any related issue.

### AI-Assisted Contributions

We welcome contributions created with the assistance of AI tools. However, all contributors must:

- **Clearly disclose AI assistance** in your pull request description and commit messages
- **Provide due diligence** by:
  - Testing the code thoroughly (run all tests with `cargo-all-features`, see above)
  - Reviewing the generated code for correctness and adherence to project conventions
  - Understanding what the code does and why it works
  - Ensuring the contribution follows the project's style and patterns
- **Take responsibility** for the final code quality and functionality

AI tools can be excellent for productivity, but human oversight and understanding remain essential for maintaining code quality.

By contributing you agree to license your contributions under MIT or Apache 2.0 licenses.
