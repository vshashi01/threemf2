//! 3MF Specification-compliant type aliases
//!
//! These types map directly to the 3MF XSD schema simple types:
//! - ST_ResourceID: Object IDs, property group IDs (1 to 2^31-1)
//! - ST_ResourceIndex: Vertex indices, property indices (0 to 2^31-1)

/// 3MF Resource ID type
/// XSD: ST_ResourceID (xs:positiveInteger, maxExclusive="2147483648")
/// Used for: object IDs, property group IDs, material IDs
pub type ResourceID = u32;

/// 3MF Resource Index type
/// XSD: ST_ResourceIndex (xs:nonNegativeInteger, maxExclusive="2147483648")
/// Used for: vertex indices (v1, v2, v3), property indices (p1, p2, p3, pindex)
pub type ResourceIndex = u32;

/// Optional Resource ID
/// Used for optional pid attributes
pub type OptionalResourceID = Option<ResourceID>;

/// Optional Resource Index
/// Used for optional pindex, p1, p2, p3 attributes
pub type OptionalResourceIndex = Option<ResourceIndex>;

/// Extension trait for converting ResourceIndex to usize
pub trait ResourceIndexExt {
    /// Convert to usize for array indexing
    fn as_usize(self) -> usize;
}

impl ResourceIndexExt for ResourceIndex {
    #[inline]
    fn as_usize(self) -> usize {
        self as usize
    }
}

/// Extension trait for converting ResourceID to usize
pub trait ResourceIDExt {
    /// Convert to usize for array indexing
    fn as_usize(self) -> usize;
}

impl ResourceIDExt for ResourceID {
    #[inline]
    fn as_usize(self) -> usize {
        self as usize
    }
}
