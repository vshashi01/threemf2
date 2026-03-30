# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed

- **Core types have been changed for better memory usage**
  - Changed all uses of `usize` for resource ID with a custom `ResourceId` type alias with u32
  - Changed all uses of `usize` for resource index with a custom `ResourceIndex` type alias with u32
  - Changed all uses of `Option<usize>` for optional resource Ids with a custom `OptionalResourceId` type backed by u32
  - Change all uses if `Option<usize>` for optional resource index with a custom `OptionalResourceIndex` type backed by u32
  - `Object` struct is changed to have a field `kind` of type `ObjectKind` where the specific geometry is a variant inside type `ObjectKind`. This ensures future extensions to `ObjectKind` to (hopefully) not be breaking change.

- **Some fields with f64 have been changed for faster deserialization**
  - Uses of f64 in the `Mesh` struct has been updated to use a new type Double with deserialization based on `lexical::f64` which improves performance significantly.

- **Capacity based Vec handling for Vertices and Triangles**
  - When deserializing `Mesh` with `memory-optimized-read` now the `Vertex` and `Triangle` collections are allocated in larger chunks to reduce the memory reallocation overhead for very large meshes.

### Added

- **Boolean Operations Extension Support**
  - Added `BooleanShape`, `Boolean`, and `BooleanOperation` types to `core::boolean` module
  - Added XML serialization/deserialization support (ToXml/FromXml) for boolean operations
  - Added namespace definitions: `BOOLEAN_NS` and `BOOLEAN_PREFIX` in `threemf_namespaces`
  - Added `ObjectKind::BooleanShape` variant to support boolean shape objects
  - Added example: `examples/boolean.rs` demonstrating boolean operations usage
  - Added `BooleanShapeRef` reference type for querying boolean shape objects
  - Added `BooleanRef` reference type for boolean operands
  - Added `get_boolean_shape_objects()` to query boolean shapes from packages
  - Added `get_boolean_shape_objects_from_model()` for model-level queries
  - Added helper methods: `is_union()`, `is_difference()`, `is_intersection()`
  - Added `base_objectid()`, `operation()`, and `booleans()` accessors
  - Added `BooleanObjectBuilder` for creating boolean shape objects
  - Added `BooleanShapeBuilder` for configuring base objects and operations
  - Added `BooleanBuilder` for configuring boolean operands

- **Added Validator**
  - A new io::validator functionality is added to allow users to validate the 3MF Model or 3MF Package against some rules.
  - Currently only supports a limited set of rules. This is meant for future extension.

### Fixed

- Fixed namespace handling
  - Namespaces were previously handled wrongly when writing 3MF packages where the required-extensions field was set wrongly leading to files not readable in specific readers.

### Commit Statistics

<csr-read-only-do-not-edit/>

- 22 commits contributed to the release.
- 0 commits were understood as [conventional](https://www.conventionalcommits.org).
- 7 unique issues were worked on: [#22](https://github.com/vshashi01/threemf2/issues/22), [#24](https://github.com/vshashi01/threemf2/issues/24), [#25](https://github.com/vshashi01/threemf2/issues/25), [#26](https://github.com/vshashi01/threemf2/issues/26), [#27](https://github.com/vshashi01/threemf2/issues/27), [#28](https://github.com/vshashi01/threemf2/issues/28), [#29](https://github.com/vshashi01/threemf2/issues/29)

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

- **[#22](https://github.com/vshashi01/threemf2/issues/22)**
  - Reorganize to workspace ([`59062cd`](https://github.com/vshashi01/threemf2/commit/59062cd2d495d88416f63b379078420639dd235e))
- **[#24](https://github.com/vshashi01/threemf2/issues/24)**
  - Speed up ([`fdac242`](https://github.com/vshashi01/threemf2/commit/fdac242d72b55d656e56f4cec1fbb2070fe05531))
- **[#25](https://github.com/vshashi01/threemf2/issues/25)**
  - Xsd validation ([`32c00c3`](https://github.com/vshashi01/threemf2/commit/32c00c3f2de42dadb62e8e4ea88c91ac7e88241c))
- **[#26](https://github.com/vshashi01/threemf2/issues/26)**
  - Validator ([`888763b`](https://github.com/vshashi01/threemf2/commit/888763baff5bf1b5f1739d69ce51f0c478e1fcf))
- **[#27](https://github.com/vshashi01/threemf2/issues/27)**
  - Thumbnail generator ([`d1e213d`](https://github.com/vshashi01/threemf2/commit/d1e213da7c82cd971d6986e9114affc7f951396c))
- **[#28](https://github.com/vshashi01/threemf2/issues/28)**
  - Update-deps ([`8bd1ecb`](https://github.com/vshashi01/threemf2/commit/8bd1ecb2d45f42a096789ab7da263a4ef2b3191f))
- **[#29](https://github.com/vshashi01/threemf2/issues/29)**
  - Add Boolean shape extension support ([`1ea1cd2`](https://github.com/vshashi01/threemf2/commit/1ea1cd25af0de910f4ad910dde0bba9f51bc0a19))
- **Uncategorized** - Fixed the issue with xmlns missed in the namespace definitions for the model element. ([`b03ca18`](https://github.com/vshashi01/threemf2/commit/b03ca1802d95ea2d5ac53834ac31ef3a5ca0a363)) - Fixed depedency and atttribute issue when only speed-optimized-read is enabled ([`5603a9a`](https://github.com/vshashi01/threemf2/commit/5603a9ab698293aff295c584ad7f9e0b1290c0e8)) - Adjusting changelogs prior to release of threemf2 v0.1.2 ([`2839d6e`](https://github.com/vshashi01/threemf2/commit/2839d6e80af0926c10eb21c9bbcec524e98452f5)) - Update version ([`f0a51e8`](https://github.com/vshashi01/threemf2/commit/f0a51e8ff3ee9c0deada4f99c28fd6f21d5f3a19)) - Adjusting changelogs prior to release of threemf2 v0.1.1 ([`d405cd7`](https://github.com/vshashi01/threemf2/commit/d405cd734396349d21899cd2148b5ad6e28bfbd2)) - Update changelog for updating examples ([`88533b4`](https://github.com/vshashi01/threemf2/commit/88533b41b99d585b27abd8658d3e3ac8febda019)) - Readded required features to examples ([`870b1df`](https://github.com/vshashi01/threemf2/commit/870b1dfcdf3546d4ecf05a686eb4c5711533f354)) - Adjusting changelogs prior to release of threemf2 v0.1.1 ([`46c0322`](https://github.com/vshashi01/threemf2/commit/46c03226ae8a31d3754164c56e7e95eabcdf68af)) - Update readme ([`6e7e937`](https://github.com/vshashi01/threemf2/commit/6e7e9375c31af0d99458a8ea378b84398ffcf434)) - Fixed Readme ([`1ed1e3c`](https://github.com/vshashi01/threemf2/commit/1ed1e3cfc7a59e2840353478443d868d71ccb1cb)) - Adjusting changelogs prior to release of threemf2 v0.1.0 ([`e1457c4`](https://github.com/vshashi01/threemf2/commit/e1457c4f407df4b85e1910e7dc4be4fbdb904b4e)) - Adjusting changelogs prior to release of threemf2 v0.1.0 ([`7690689`](https://github.com/vshashi01/threemf2/commit/76906891365ebf0cd632d2d2f8eb4e4b1fab77cc)) - Update changelog ([`8b22a39`](https://github.com/vshashi01/threemf2/commit/8b22a39ebc7258ffc5df57f11e93fbfd86dfe732)) - Adjusting changelogs prior to release of threemf2 v0.1.0 ([`1a2ecd5`](https://github.com/vshashi01/threemf2/commit/1a2ecd50d22590564e9f2a7ac2c8fd4ef979f514)) - Clean up of toml file ([`36e84d7`](https://github.com/vshashi01/threemf2/commit/36e84d7b0ee127ef30d9eb51bf21da6c6779832d))
</details>

## 0.1.2 (2025-11-30)

## 0.1.1 (2025-11-30)

## v0.1.0 (2025-11-30)
