# Architecture of `threemf2`

This document describes the high-level architecture of the `threemf2` crate. It is intended for **new contributors** who want to understand how the crate is organized, why specific design decisions were made, and how to navigate the codebase.

---

## 1. Overview & Design Goals

The `threemf2` crate is the main library in the workspace. It provides read and write support for the 3MF (3D Manufacturing Format) file format, which is a ZIP-based container format used for 3D printing workflows.

**Primary design goals:**

- **Spec compliance**: Correctly parse and write all 3MF core specification elements and extensions. The library is designed to handle real-world 3MF files from CAD tools and slicers.
- **Memory efficiency**: 3MF files can contain large triangle meshes. The default configuration optimizes for lower memory usage during parsing.
- **Feature flexibility**: Not all users need the same capabilities. I/O operations and serialization strategies are behind Cargo feature flags so consumers only compile what they need.
- **Ergonomics**: Provide stable, easy-to-use APIs for inspecting data (`query` modules) and building models (`builder` modules).
- **Safety**: The crate explicitly forbids unsafe code. All serialization, ZIP I/O, and data processing is done with safe Rust abstractions.

**The crate is part of a workspace** that includes `threemf2-benches` (performance benchmarks), `threemf2-tests` (integration tests with large data files), and `threemf2-thumbnail` (thumbnail utilities). This document covers only the `threemf2` crate itself.

---

## 2. High-Level Module Structure

The crate is divided into three top-level modules:

```
src/
├── lib.rs                              # Crate entry point, feature-gated module exports
├── threemf_namespaces.rs               # Namespace definitions for all supported 3MF extensions
├── model/                              # Core 3MF data model
│   ├── mod.rs                          # Public re-exports
│   ├── domain/                         # Raw XML-mapped types (20+ files)
│   ├── query.rs                        # Stable read-only views (ModelView, ObjectView, etc.)
│   └── builder.rs                      # Programmatic construction of domain types (ModelBuilder, etc.)
└── package/                            # ZIP-based package I/O
    ├── mod.rs                          # Public re-exports
    ├── threemf_package.rs              # Eager loading (all data upfront)
    ├── threemf_package_lazy_reader.rs  # Lazy loading (on-demand)
    ├── query.rs                        # Package-level iteration (cross-model queries)
    ├── builder.rs                      # Programmatic construction of ThreemfPackage
    ├── domain/                         # ZIP internals (content types, relationships, validators)
    └── error.rs                        # Unified error type
```

### 2.1. Module responsibilities

| Module               | Responsibility                                                                                                                                           |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `threemf_namespaces` | Defines all XML namespace URIs and prefixes used by the 3MF core spec and extensions. Provides the `ThreemfNamespace` enum.                              |
| `model`              | The **data layer**. Maps 3MF XML elements to Rust types. Contains serialization/deserialization logic, query APIs, and builder APIs.                     |
| `package`            | The **I/O layer**. Reads and writes ZIP archives containing 3MF model files, thumbnails, and relationships. Only compiled when I/O features are enabled. |

---

## 3. Core Data Layer (`model`)

### 3.1. Domain types (`model/domain/`)

The `domain/` directory contains the raw Rust types that map directly to the 3MF XML schema. These are the building blocks of the 3MF data model:

### 3.2. Serialization derives

Domain types use feature-gated derive macros so that the same struct can be serialized with different XML backends:

```rust
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "memory-optimized-read", derive(FromXml))]
#[cfg_attr(feature = "write", derive(ToXml))]
#[derive(Debug, Clone, PartialEq)]
```

- **`write` feature** → adds `ToXml` derive via `instant-xml` (serializes to XML string).
- **`memory-optimized-read` feature** → adds `FromXml` derive via `instant-xml` (streaming XML parser, lower memory).
- **`speed-optimized-read` feature** → adds `serde::Deserialize` derive via `serde-roxmltree` (DOM-based parser, faster but higher memory). **Deprecated.**

This pattern allows the same data model to work with multiple )de)serialization backends without duplicating the struct definitions.

### 3.3. Query views (`model/query.rs`)

The raw domain types are faithful to the XML structure, which makes them stable for serialization but sometimes awkward for consumers. The **query view pattern** provides a stable, read-only API layer over the raw types.

**Why views matter**: If the internal domain types change (e.g., adding new fields, renaming private members), the public query API can remain stable. Contributors should add new public-facing query methods here rather than exposing raw domain structs directly.

### 3.4. Builder API (`model/builder.rs`)

The builder API provides a programmatic way to construct 3MF models without manually managing IDs, extensions, or namespace declarations.

**Design notes**:

- Object IDs are automatically managed by `ModelBuilder`. Consumers use type-safe `ObjectId` handles.
- The builder automatically detects and adds required extensions (e.g., Beam Lattice, Boolean Operations, Production) based on what features are used.

---

## 4. Package I/O Layer (`package`)

The `package` module handles the physical ZIP archive format that 3MF files use. This module is **feature-gated** and only compiled when an I/O feature is enabled.

### 4.1. Eager loading: `ThreemfPackage`

`ThreemfPackage` (`threemf_package.rs`) loads the entire 3MF file into memory upfront:

- **Reading**: The `from_reader_with_memory_optimized_deserializer()` method reads the ZIP archive, parses `[Content_Types].xml`, discovers all relationship files, and loads all models, thumbnails, and unknown parts into the respective `HashMap`s.
- **Writing**: The `write()` method writes a well-formed ZIP archive with all parts, relationship files, and content types.

### 4.2. Lazy loading: `ThreemfPackageLazyReader`

`ThreemfPackageLazyReader` (`threemf_package_lazy_reader.rs`) is designed for memory-constrained environments or when you only need to inspect package metadata without loading all data.

**How it works**:

- On construction, it only parses `[Content_Types].xml` and relationship files.
- Models, thumbnails, and unknown parts are loaded **on-demand** via `with_model()`, `with_thumbnail()`, and `with_unknown_part()` callbacks.
- The `CachePolicy` controls whether loaded data is cached:
  - `CacheAll` — caches after first access (good for repeated access).
  - `NoCache` — re-reads from ZIP each time (best for memory-constrained, one-pass patterns).
- Raw XML access is also available via `with_model_xml()`, `with_relationships_xml()`, and `with_content_types_xml()` for string extraction without deserialization.

### 4.3. Package query API (`package/query.rs`)

The package-level query API extends the model-level query views to work across **all models** in a package (root + sub-models).

Examples:

- `get_models()` → iterates over all models with path information.
- `get_objects()` → iterates over all objects across all models.
- `get_mesh_objects()` → iterates over all mesh objects across all models.
- `get_items()` → iterates over all build items across all models.
- `get_slice_stacks()` → iterates over all slice stacks across all models.

Each result includes an `origin_model_path` field indicating which model file the data came from.

### 4.4. Package builder (`package/builder.rs`)

`ThreemfPackageBuilder` assembles multiple models into a single 3MF package with correct ZIP structure, content types, and relationships.

**Responsibilities**:

- Validates that root and sub-model paths are unique and well-formed.
- Automatically generates content types and relationship files.
- Handles thumbnails and unknown parts (custom OPC parts).

### 4.5. ZIP internals (`package/domain/`)

- **`content_types.rs`** — `[Content_Types].xml` representation. Tracks default content types by file extension.
- **`relationship.rs`** — OPC relationship files. Tracks which part depends on which other part.
- **`thumbnail_handle.rs`** — Thumbnail image data with format detection (PNG, JPEG).
- **`zip_utils.rs`** — ZIP archive helpers: read ZIP entries to strings, discover relationship files, dispatch to XML deserializers.
- **`validator.rs` / `validator_rules.rs`** — Package validation rules.
- **`utils.rs`** — General XML utility functions (e.g., parsing `xmlns` attributes).

### 4.6. Error handling (`package/error.rs`)

The `Error` enum is a unified error type for all package I/O operations, using `thiserror`:

---

## 5. Serialization Strategy

The crate supports two XML serialization backends, selected at compile time via feature flags.

### 5.1. `instant-xml` (default, memory-optimized)

- **Backend**: `instant-xml` crate (version 0.7.5).
- **Strategy**: Streaming XML parser. Reads XML incrementally without building a full DOM tree.
- **Pros**: Lower peak memory usage. Suitable for large files.
- **Cons**: Slightly slower than DOM-based parsing.
- **Feature**: `memory-optimized-read` (for deserialization), `write` (for serialization).

### 5.2. `serde-roxmltree` (deprecated, speed-optimized)

- **Backend**: `serde-roxmltree` crate + `serde::Deserialize`.
- **Strategy**: DOM-based parsing. Builds the entire XML tree in memory before deserialization.
- **Pros**: Faster parsing for small to medium files.
- **Cons**: Higher memory usage. Deprecated because the performance gain does not justify the memory cost for most 3MF workflows.
- **Feature**: `speed-optimized-read`.

### 5.3. Why both exist

3MF files vary widely in size. A simple test cube is a few kilobytes; a complex architectural model with millions of triangles can be hundreds of megabytes. The dual-backend design lets users choose the right trade-off for their use case. The speed-optimized backend is maintained for backward compatibility but is not recommended for new code.

### 5.4. Feature gating pattern

The crate uses `#[cfg_attr(feature = "...", derive(...))]` to attach serialization derives conditionally. This keeps the domain type definitions clean and avoids duplicating structs for each backend.

---

## 6. Extension Handling Philosophy

### 6.1. Always compiled

3MF extensions (Beam Lattice, Production, Materials, Slice, Boolean Operations, Displacement, Triangle Set) are **never** behind Cargo feature flags. They are always compiled into the crate. This is a deliberate design choice: 3MF is a specification-driven format, and the library must be able to read and write any valid 3MF file by default.

### 6.2. Namespace tracking

The `Model` struct tracks which extensions are actually used in a specific model via `used_namespaces()`:

This is used during **writing** to include only the necessary `xmlns` declarations in the `<model>` tag, keeping the XML clean and valid.

### 6.3. Unknown extensions

For forward compatibility, the library supports `ThreemfNamespace::Unknown { prefix, uri }`. If a 3MF file uses an extension that the library does not yet recognize, the namespace is preserved during reading and writing, but the actual XML elements are not parsed into typed Rust structures.

---

## 7. Compact Type System

The 3MF specification defines many integer IDs and indices. The crate uses custom types to minimize memory usage and enforce spec constraints:

| Type                      | Description                          | Memory optimization                                                                  |
| ------------------------- | ------------------------------------ | ------------------------------------------------------------------------------------ |
| `ResourceId`              | `u32` alias for object/property IDs  | Standard size                                                                        |
| `OptionalResourceId`      | Optional `ResourceId`                | `Option<NonZeroU32>` — 4 bytes instead of 8                                          |
| `OptionalResourceIndex`   | Optional `ResourceIndex`             | Sentinel `u32` (`u32::MAX`) — 4 bytes instead of 8                                   |
| `StrResource`             | String value                         | `CompactString` — inline small strings, no heap allocation for ≤24 bytes             |
| `PathResource`            | Package-internal path                | Validated, normalized path with spec-compliant rules                                 |
| `Double`                  | `f64` newtype                        | Uses `lexical-core` for fast parsing, avoids `std::str::parse` overhead              |
| `Color`                   | sRGB color                           | 4 bytes (`u8` x 4), supports `#RRGGBB` and `#RRGGBBAA`                               |
| `UuidResource`            | UUID string                          | Optional `uuid` crate feature for validation; stores as string when feature disabled |
| `ResourceIdCollection`    | Space-delimited `ResourceId` list    | Thin wrapper over `Vec<u32>` with custom XML serialization                           |
| `ResourceIndexCollection` | Space-delimited `ResourceIndex` list | Thin wrapper over `Vec<u32>` with custom XML serialization                           |

These types are defined in `model/domain/types.rs` and are reused throughout the domain model.

---

## 8. Feature Flag Architecture

The crate uses Cargo features to control compilation of I/O operations and serialization backends.

### 8.1. Feature categories

| Category          | Features                                                                                         | Description                                            |
| ----------------- | ------------------------------------------------------------------------------------------------ | ------------------------------------------------------ |
| **Serialization** | `write`, `memory-optimized-read`, `speed-optimized-read`                                         | Controls which XML derive macros are available.        |
| **Package I/O**   | `package-write`, `package-memory-optimized-read`, `io-speed-optimized-read`, `package-lazy-read` | Controls which ZIP I/O types and methods are compiled. |
| **Utility**       | `uuid`                                                                                           | Enables UUID validation for Production extension data. |

### 8.2. Default features

```toml
default = [
    "package-write",
    "package-memory-optimized-read",
    "package-lazy-read",
    "write",
    "memory-optimized-read",
]
```

This gives users a full-featured crate with eager + lazy reading and writing out of the box.

### 8.3. Feature dependencies

```
package-write                    # requires write
package-memory-optimized-read    # requires memory-optimized-read
io-speed-optimized-read          # requires speed-optimized-read
package-lazy-read                # requires package-memory-optimized-read
```

### 8.4. Gating pattern

The crate uses `#[cfg(any(feature = "...", ...))]` extensively:

- On `mod` declarations to hide entire modules.
- On `impl` blocks to hide methods.
- On `pub use` re-exports to hide public types.

This ensures that the crate compiles cleanly with minimal feature sets and that users do not see APIs they cannot use.

---

## 9. Safety Policy

The crate explicitly forbids unsafe code. This is enforced at the crate root:

```rust
#![forbid(unsafe_code)]
```

This is a hard architectural boundary. All serialization, ZIP I/O, XML parsing, and data processing is implemented with safe Rust abstractions. No unsafe code is permitted or required.

---

## 10. Testing & Quality

### 10.1. Unit tests

Tests are embedded in the source files using `#[cfg(test)]` modules. This keeps tests close to the code they exercise. For example:

- `model/domain/model.rs` contains `mod write_tests` and `mod memory_optimized_read_tests`.
- `package/threemf_package.rs` contains tests for reading, writing, round-tripping, and namespace tracking.
- `package/threemf_package_lazy_reader.rs` contains tests for lazy loading, caching, and string extraction.

### 10.2. Integration tests

Integration tests are in the separate `threemf2-tests` crate. They use real-world 3MF files (some stored in Git LFS) and exercise end-to-end workflows: core I/O, production extension, beam lattice, roundtrip verification, and third-party file compatibility.

### 10.3. CI requirements

- All code must pass `cargo fmt --all -- --check`.
- All code must pass `cargo clippy --all-targets --all-features -- -D warnings` (warnings are treated as errors).
- Tests should run with `cargo test --all-features`.

### 10.4. Benchmarks

Performance benchmarks are in `threemf2-benches`. They compare the different serialization backends (`instant-xml` vs `serde-roxmltree`) and measure package reader/writer performance.

---

## 11. Code Conventions

- **Imports**: Grouped as external crates → internal modules → `std`.
- **Feature gates**: Use `#[cfg(feature = "...")]` on imports and modules.
- **Naming**: `snake_case` for files and functions, `PascalCase` for types and enums, `SCREAMING_SNAKE_CASE` for constants.
- **Documentation**: All public APIs must have doc comments (`///`). File paths are referenced as `[Type](src/path/file.rs)`.
- **Error handling**: Use `thiserror` for error types. Prefer `Result` over panics for recoverable errors.
- **No doc tests**: The project does not add runnable doc tests.
- **Tests**: Use `pretty_assertions` for better test failure messages.

---

## 12. How to Navigate the Codebase

**If you want to...**

- **Add a new 3MF extension** → Add domain types in `model/domain/`, add namespace to `threemf_namespaces.rs`, update `Model::used_namespaces()`, add query views in `model/query.rs`, add builder support in `model/builder.rs`.
- **Change serialization behavior** → Look at `#[cfg_attr(...)]` derives in the domain types. The serialization logic lives in the `instant-xml` and `serde-roxmltree` crates, not in this crate.
- **Add a new package I/O feature** → Feature-gate the new code in `package/mod.rs`, add methods to `ThreemfPackage` or `ThreemfPackageLazyReader`, add tests in the same file.
- **Add a new query API** → Add the view struct in `model/query.rs` (or `package/query.rs` for cross-model queries), add a public function that returns it.
- **Fix a bug in type parsing** → Look at `model/domain/types.rs` for the custom type definitions (`Double`, `Color`, `OptionalResourceId`, etc.).
- **Understand the ZIP format handling** → Read `package/domain/zip_utils.rs`, `content_types.rs`, and `relationship.rs`.
- **Add a builder feature** → Extend `model/builder.rs` or `package/builder.rs`. Follow the existing builder pattern with `&mut self` chaining and validation at `build()` time.

---

## 13. Related Documents

- [`README.md`](README.md) — User-facing documentation, feature flag guide, examples.
- [`AGENTS.md`](AGENTS.md) — Development guide for coding agents: build commands, test commands, code style.
- [`Changelog.md`](Changelog.md) — Release history (Keep a Changelog format).
- [3MF Specification](https://3mf.io/specification/) — Official 3MF specification.
- [Open Packaging Conventions](https://standards.iso.org/ittf/PubliclyAvailableStandards/c061796_ISO_IEC_29500-2_2012.zip) — ZIP/OPC format specification.
