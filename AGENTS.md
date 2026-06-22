# AGENTS.md - Development Guide for Coding Agents

## Build & Test Commands

- **Build**: `cargo build --all-features`
- **Test all**: `cargo test --all-features`
- **Test lazy reader**: `cargo test --features io-lazy-read`
- **Run benchmarks**: `cargo bench --features "io-memory-optimized-read,io-speed-optimized-read"` (speed-optimized-read is deprecated)
- **Run examples**: `cargo run --example write --features package-write`
- **Lint/Format check**: `cargo fmt --all -- --check && cargo clippy --all-targets --all-features -- -D warnings`
- **Format code**: `cargo fmt --all`
- **MSRV**: 1.89.0 (Rust 2024 edition)

## Code Style & Conventions

- **Imports**: Group external crates, internal modules, and then std; use feature-gated imports (`#[cfg(feature = "...")]`)
- **Features**: Extensive feature flags for I/O operations and XML strategies; gate code with `#[cfg(feature = "...")]`
- **3MF Extensions**: Current and future 3MF extensions are not gated as features - they are always available
- **Derive macros**: Feature-gate serialization derives (`#[cfg_attr(feature = "write", derive(ToXml))]`, etc.)
- **Lazy Loading**: Use `with_model()` pattern for accessing models; prefer `CachePolicy::NoCache` for memory-constrained environments
- **Error handling**: Use `thiserror` for error types; see `src/io/error.rs` for the Error enum pattern
- **Naming**: Snake_case for files/functions, PascalCase for types/enums, SCREAMING_SNAKE_CASE for constants
- **Tests**: Integration tests in `tests/` with feature gates; use `pretty_assertions` for test assertions; no doc tests are added
- **Documentation**: Add doc comments (`///`) for public APIs; reference file paths as `[Type](src/path/file.rs)`
- **Changelog**: Maintain Changelog.md following [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format; update for each release with categorized changes (Added, Changed, Fixed, etc.)
- **Clippy**: Code must pass `clippy --all-targets --all-features -- -D warnings` (warnings are errors in CI)

## Project Structure

- `src/core/`: Core 3MF data structures (model, object, mesh, resources, transform, beamlattice, etc.)
- `src/io/`: Package I/O operations (ThreemfPackage, ThreemfPackageLazyReader, query helpers, ZIP utilities)
- `tests/`: Integration tests (core_io, production_io, beamlattice_io, roundtrip, third_party_read)
- `examples/`: Feature-specific examples (write, unpack, memory/speed-optimized reads, etc.)
- `benches/`: Performance benchmarks

## Feature Flags Overview

- `io-*`: Package I/O operations (write, memory-optimized-read, speed-optimized-read [deprecated], lazy-read)
- `*-optimized-read`: XML deserialization strategies (memory vs speed trade-offs)
- `io-lazy-read`: Lazy loading functionality (defers loading until accessed)
- Default: `package-write`, `io-memory-optimized-read`, `io-lazy-read`, `write`, `memory-optimized-read`
- **Note**: 3MF extensions (beam lattice, production, etc.) are always available regardless of feature flags

## API Overview

### Core Data Structures

- `Model`: Root 3MF model with resources and build configuration
- `Object`: 3D objects (meshes or component assemblies)
- `Mesh`: Triangle mesh geometry
- `Component`: Object references with transforms

### Package I/O

- `ThreemfPackage`: Eager loading - loads all data upfront
- `ThreemfPackageLazyReader`: Lazy loading - loads metadata first, data on-demand
- Query functions in `io::query` for inspecting loaded packages

### Usage Patterns

```rust
// Eager loading
let package = ThreemfPackage::from_reader_with_memory_optimized_deserializer(reader, unpack_sub_models)?;

// Lazy loading
let package = ThreemfPackageLazyReader::from_reader_with_memory_optimized_deserializer(reader, CachePolicy::NoCache)?;
package.with_model("path/to/model.model", |model| {
    // Work with model
})?;
```

## Testing Strategy

- **Integration tests**: End-to-end functionality in `tests/`
- **Third-party compatibility**: Real-world 3MF file testing
- **Roundtrip testing**: Write → read verification
- **Feature gating**: Tests gated by required features
- **Performance**: Benchmarks for regression detection
- **No doc tests**: Documentation examples are not tested as runnable code
