pub mod domain;

/// Query helpers for model-level data.
pub mod query;

pub use domain::types::{
    Color, Double, OptionalResourceId, OptionalResourceIndex, PathResource, PathResourceError,
    ResourceId, ResourceIdCollection, ResourceIndex, ResourceIndexCollection, StrResource,
    UuidResource,
};

pub mod builder;
