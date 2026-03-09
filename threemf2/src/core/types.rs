//! 3MF Specification-compliant type aliases
//!
//! These types map directly to the 3MF XSD schema simple types:
//! - ST_ResourceID: Object IDs, property group IDs (1 to 2^31-1)
//! - ST_ResourceIndex: Vertex indices, property indices (0 to 2^31-1)

#[cfg(feature = "write")]
use instant_xml::{Id, Serializer, ToXml};

#[cfg(feature = "memory-optimized-read")]
use instant_xml::{Error, FromXml, Kind};

#[cfg(feature = "speed-optimized-read")]
use serde::Deserialize;

/// 3MF Resource ID type
/// XSD: ST_ResourceID (xs:positiveInteger, maxExclusive="2147483648")
/// Used for: object IDs, property group IDs, material IDs
pub type ResourceId = u32;

/// 3MF Resource Index type
/// XSD: ST_ResourceIndex (xs:nonNegativeInteger, maxExclusive="2147483648")
/// Used for: vertex indices (v1, v2, v3), property indices (p1, p2, p3, pindex)
pub type ResourceIndex = u32;

/// Optional Resource ID
/// Used for optional pid attributes
pub type OptionalResourceId = Option<ResourceId>;

/// Sentinel value representing "None" for OptionalResourceIndex
const OPTIONAL_RESOURCE_INDEX_NONE: u32 = u32::MAX;

/// Compact Optional type for ResourceIndex (4 bytes vs 8 bytes for Option<u32>)
/// Uses sentinel value instead of enum discriminant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
pub struct OptionalResourceIndex(u32);

impl OptionalResourceIndex {
    /// Create from raw u32 value
    /// - Sentinel (u32::MAX) → None
    /// - Valid index (0-2_147_483_647) → Some
    /// - Out of range (>2_147_483_647, not sentinel) → panic
    pub fn new(value: u32) -> Self {
        if value == OPTIONAL_RESOURCE_INDEX_NONE {
            Self::none()
        } else {
            assert!(
                value <= 2_147_483_647,
                "ResourceIndex {} exceeds 3MF spec limit (2,147,483,647)",
                value
            );
            Self(value)
        }
    }

    /// Create None value
    pub const fn none() -> Self {
        Self(OPTIONAL_RESOURCE_INDEX_NONE)
    }

    /// Check if None
    pub fn is_none(&self) -> bool {
        self.0 == OPTIONAL_RESOURCE_INDEX_NONE
    }

    /// Check if Some
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    /// Get the value
    pub fn get(&self) -> Option<ResourceIndex> {
        if self.is_none() { None } else { Some(self.0) }
    }

    /// Unwrap with default
    pub fn unwrap_or(&self, default: ResourceIndex) -> ResourceIndex {
        self.get().unwrap_or(default)
    }
}

impl Default for OptionalResourceIndex {
    fn default() -> Self {
        Self::none()
    }
}

#[cfg(feature = "write")]
impl ToXml for OptionalResourceIndex {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _field: Option<Id<'_>>,
        serializer: &mut Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        if self.is_some() {
            let value = self.0.to_string();
            serializer.write_str(&value)?;
        }

        Ok(())
    }

    fn present(&self) -> bool {
        self.is_some()
    }
}

#[cfg(feature = "memory-optimized-read")]
impl<'xml> FromXml<'xml> for OptionalResourceIndex {
    #[inline]
    fn matches(id: instant_xml::Id<'_>, field: Option<instant_xml::Id<'_>>) -> bool {
        // Match if the attribute name matches the field name
        if let Some(field_id) = field {
            id == field_id
        } else {
            false
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut instant_xml::Deserializer<'cx, 'xml>,
    ) -> Result<(), Error> {
        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        let value = match deserializer.take_str()? {
            Some(value) => {
                #[cfg(feature = "memory-optimized-read-experimental")]
                let value: u32 = lexical_core::parse(value.as_bytes())
                    .map_err(|_| Error::MissingValue("Failed to parse OptionalResourceIndex"))?;

                #[cfg(all(
                    feature = "memory-optimized-read",
                    not(feature = "memory-optimized-read-experimental")
                ))]
                let value: u32 = value
                    .parse()
                    .map_err(|_| Error::MissingValue("Failed to parse OptionalResourceIndex"))?;

                Self::new(value)
            }
            None => Self::none(),
        };

        *into = value;
        Ok(())
    }

    type Accumulator = Self;
    const KIND: Kind = instant_xml::Kind::Scalar;
}

#[cfg(feature = "memory-optimized-read")]
impl instant_xml::Accumulate<OptionalResourceIndex> for OptionalResourceIndex {
    fn try_done(self, _: &'static str) -> Result<OptionalResourceIndex, Error> {
        Ok(self)
    }
}

#[cfg(feature = "speed-optimized-read")]
pub mod serde_impl {
    use super::OptionalResourceIndex;
    use serde::{Deserialize, Deserializer};

    /// Returns default none() value for serde default attribute
    pub fn default_none() -> OptionalResourceIndex {
        OptionalResourceIndex::none()
    }

    /// Custom deserializer for OptionalResourceIndex
    /// - Missing XML attribute → none()
    /// - Present XML attribute → parse as u32 and validate
    pub fn deserialize<'de, D>(deserializer: D) -> Result<OptionalResourceIndex, D::Error>
    where
        D: Deserializer<'de>,
    {
        // roxmltree with serde: missing attribute = None, present = Some(u32)
        let opt: Option<u32> = Option::deserialize(deserializer)?;

        match opt {
            None => Ok(OptionalResourceIndex::none()),
            Some(v) => Ok(OptionalResourceIndex::new(v)),
        }
    }
}

impl From<Option<u32>> for OptionalResourceIndex {
    fn from(value: Option<u32>) -> Self {
        match value {
            Some(val) => Self::new(val),
            None => Self::none(),
        }
    }
}

impl From<OptionalResourceIndex> for Option<u32> {
    fn from(value: OptionalResourceIndex) -> Self {
        if value.is_some() { value.get() } else { None }
    }
}

pub trait IntoIndex {
    fn into(self) -> usize;
}

impl IntoIndex for u32 {
    #[inline]
    fn into(self) -> usize {
        self as usize
    }
}
