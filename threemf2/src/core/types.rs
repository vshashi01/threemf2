//! 3MF Specification-compliant type aliases or New types
//!
//! These types map directly to the 3MF XSD schema simple types:
//! - ResourceId = ST_ResourceID: Object IDs, property group IDs (1 to 2^31-1)
//! - ResourceIndex = ST_ResourceIndex: Vertex indices, property indices (0 to 2^31-1)
//! - Double = ST_Number: All number inputs in the form of 64-byte float number

use std::num::NonZeroU32;

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

/// Compact Optional type for ResourceId with [`Option<NonZeroU32>`]
/// (4 bytes vs 8 bytes for [`Option<u32>`])
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
pub struct OptionalResourceId(Option<NonZeroU32>);

impl OptionalResourceId {
    /// Create from u32 value
    /// - 0 → None
    /// - 1..=2_147_483_647 → Some(NonZeroU32)
    /// - > 2_147_483_647 → panic
    pub fn new(value: u32) -> Self {
        if value == 0 {
            Self::none()
        } else {
            assert!(
                value <= 2_147_483_647,
                "ResourceId {} exceeds 3MF spec limit (2,147,483,647)",
                value
            );
            Self(Some(NonZeroU32::new(value).unwrap()))
        }
    }

    pub const fn none() -> Self {
        Self(None)
    }
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }
    pub fn get(&self) -> Option<u32> {
        self.0.map(|nz| nz.get())
    }
    pub fn unwrap_or(&self, default: u32) -> u32 {
        self.get().unwrap_or(default)
    }
}

impl Default for OptionalResourceId {
    fn default() -> Self {
        Self::none()
    }
}

impl From<&[u8]> for OptionalResourceId {
    fn from(value: &[u8]) -> Self {
        match lexical_core::parse(value) {
            Ok(value) => OptionalResourceId::new(value),
            Err(_) => OptionalResourceId::none(),
        }
    }
}

#[cfg(feature = "write")]
impl ToXml for OptionalResourceId {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _field: Option<Id<'_>>,
        serializer: &mut Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        if let Some(id) = self.0 {
            serializer.write_str(&id.get().to_string())?;
        }
        Ok(())
    }

    fn present(&self) -> bool {
        self.is_some()
    }
}

#[cfg(feature = "memory-optimized-read")]
impl<'xml> FromXml<'xml> for OptionalResourceId {
    fn matches(id: instant_xml::Id<'_>, field: Option<instant_xml::Id<'_>>) -> bool {
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
                let value: u32 = lexical_core::parse(value.trim().as_bytes())
                    .map_err(|_| Error::MissingValue("Failed to parse OptionalResourceId"))?;

                if value == 0 {
                    return Err(Error::MissingValue("ResourceId cannot be 0"));
                }
                if value > 2_147_483_647 {
                    return Err(Error::MissingValue("ResourceId exceeds spec limit"));
                }
                Self(Some(NonZeroU32::new(value).unwrap()))
            }
            None => Self::none(),
        };

        *into = value;
        Ok(())
    }

    type Accumulator = Self;
    const KIND: Kind = Kind::Scalar;
}

#[cfg(feature = "memory-optimized-read")]
impl instant_xml::Accumulate<OptionalResourceId> for OptionalResourceId {
    fn try_done(self, _: &'static str) -> Result<OptionalResourceId, Error> {
        Ok(self)
    }
}

/// Sentinel value representing "None" for OptionalResourceIndex
const OPTIONAL_RESOURCE_INDEX_NONE: u32 = u32::MAX;

/// Compact Optional type for ResourceIndex (4 bytes vs 8 bytes for [`Option<u32>`])
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

impl From<&[u8]> for OptionalResourceIndex {
    fn from(value: &[u8]) -> Self {
        match lexical_core::parse(value) {
            Ok(value) => OptionalResourceIndex::new(value),
            Err(_) => OptionalResourceIndex::none(),
        }
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
                let value: u32 = lexical_core::parse(value.trim().as_bytes())
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
pub mod opt_res_id_impl {
    use super::OptionalResourceId;
    use serde::{Deserialize, Deserializer};

    pub fn default_none() -> OptionalResourceId {
        OptionalResourceId::none()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<OptionalResourceId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<u32> = Option::deserialize(deserializer)?;

        match opt {
            None => Ok(OptionalResourceId::none()),
            Some(0) => panic!("0 is an invalid value for Resource Id"),
            Some(v) => {
                if v > 2_147_483_647 {
                    panic!("Resource Id exceeds the expected maximum 2147483647");
                }
                Ok(OptionalResourceId::new(v))
            }
        }
    }
}

#[cfg(feature = "speed-optimized-read")]
pub mod opt_res_index_impl {
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

/// A collection of ResourceId values serialized as space-delimited string.
///
/// Used for attributes like `pids` in MultiProperties that contain multiple
/// resource IDs separated by spaces (e.g., "10 20 30").
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(from = "String"))]
pub struct ResourceIdCollection(Vec<u32>);

impl ResourceIdCollection {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Create from a vector of u32 values
    pub fn from_vec(values: Vec<u32>) -> Self {
        Self(values)
    }
}

impl From<Vec<u32>> for ResourceIdCollection {
    fn from(values: Vec<u32>) -> Self {
        Self(values)
    }
}

impl AsRef<[u32]> for ResourceIdCollection {
    fn as_ref(&self) -> &[u32] {
        &self.0
    }
}

impl std::ops::Deref for ResourceIdCollection {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<u32> for ResourceIdCollection {
    fn from_iter<I: IntoIterator<Item = u32>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[cfg(feature = "write")]
impl ToXml for ResourceIdCollection {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _field: Option<Id<'_>>,
        serializer: &mut Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        if !self.0.is_empty() {
            let value = self
                .0
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            serializer.write_str(&value)?;
        }
        Ok(())
    }

    fn present(&self) -> bool {
        !self.0.is_empty()
    }
}

#[cfg(feature = "memory-optimized-read")]
impl<'xml> FromXml<'xml> for ResourceIdCollection {
    #[inline]
    fn matches(id: instant_xml::Id<'_>, field: Option<instant_xml::Id<'_>>) -> bool {
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

        if let Some(value) = deserializer.take_str()? {
            if value.is_empty() {
                *into = Some(ResourceIdCollection::new());
            } else {
                let values: Vec<u32> = value
                    .split_whitespace()
                    .map(|s| lexical_core::parse(s.as_bytes()).unwrap_or(0))
                    .collect();
                *into = Some(ResourceIdCollection(values));
            }
        } else {
            *into = Some(ResourceIdCollection::new());
        }

        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: Kind = Kind::Scalar;
}

impl From<String> for ResourceIdCollection {
    fn from(value: String) -> Self {
        if value.is_empty() {
            Self::new()
        } else {
            value
                .split_whitespace()
                .map(|s| s.parse().unwrap_or(0))
                .collect()
        }
    }
}

/// A collection of ResourceIndex values serialized as space-delimited string.
///
/// Used for attributes like `pindices` in Multi and `matindices` in CompositeMaterials
/// that contain multiple resource indices separated by spaces (e.g., "0 1 2").
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(from = "String"))]
pub struct ResourceIndexCollection(Vec<u32>);

impl ResourceIndexCollection {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Create from a vector of u32 values
    pub fn from_vec(values: Vec<u32>) -> Self {
        Self(values)
    }
}

impl From<Vec<u32>> for ResourceIndexCollection {
    fn from(values: Vec<u32>) -> Self {
        Self(values)
    }
}

impl AsRef<[u32]> for ResourceIndexCollection {
    fn as_ref(&self) -> &[u32] {
        &self.0
    }
}

impl std::ops::Deref for ResourceIndexCollection {
    type Target = [u32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromIterator<u32> for ResourceIndexCollection {
    fn from_iter<I: IntoIterator<Item = u32>>(iter: I) -> Self {
        Self(iter.into_iter().collect())
    }
}

#[cfg(feature = "write")]
impl ToXml for ResourceIndexCollection {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _field: Option<Id<'_>>,
        serializer: &mut Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        if !self.0.is_empty() {
            let value = self
                .0
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            serializer.write_str(&value)?;
        }
        Ok(())
    }

    fn present(&self) -> bool {
        !self.0.is_empty()
    }
}

#[cfg(feature = "memory-optimized-read")]
impl<'xml> FromXml<'xml> for ResourceIndexCollection {
    #[inline]
    fn matches(id: instant_xml::Id<'_>, field: Option<instant_xml::Id<'_>>) -> bool {
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

        if let Some(value) = deserializer.take_str()? {
            if value.is_empty() {
                *into = Some(ResourceIndexCollection::new());
            } else {
                let values: Vec<u32> = value
                    .split_whitespace()
                    .map(|s| lexical_core::parse(s.as_bytes()).unwrap_or(0))
                    .collect();
                *into = Some(ResourceIndexCollection(values));
            }
        } else {
            *into = Some(ResourceIndexCollection::new());
        }

        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: Kind = Kind::Scalar;
}

impl From<String> for ResourceIndexCollection {
    fn from(value: String) -> Self {
        if value.is_empty() {
            Self::new()
        } else {
            value
                .split_whitespace()
                .map(|s| s.parse().unwrap_or(0))
                .collect()
        }
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

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "speed-optimized-read", derive(serde::Deserialize))]
/// A new type wrapping the f64 so that a custom (de)serializer can be implemented using
/// the lexical_core::f64 for better performance. This maybe renamed in the future to Number
/// to align with the 3MF Naming conventions.
/// [`From<f64>`] is implemented on this type and [`From<Double>`] is implemented for f64 for ease of use.
pub struct Double(f64);

impl Double {
    pub fn new(value: f64) -> Self {
        Self(value)
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

#[cfg(feature = "write")]
impl ToXml for Double {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _field: Option<Id<'_>>,
        serializer: &mut Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        let value = self.0.to_string();
        serializer.write_str(&value)?;

        Ok(())
    }
}

#[cfg(feature = "memory-optimized-read")]
impl<'xml> FromXml<'xml> for Double {
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

        if let Some(value) = deserializer.take_str()? {
            let value: f64 = lexical_core::parse(value.as_bytes())
                .map_err(|_| Error::MissingValue("Failed to parse f64 value of field {}"))?;

            *into = Some(Double(value));
        }

        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: Kind = instant_xml::Kind::Scalar;
}

impl From<f64> for Double {
    fn from(value: f64) -> Self {
        Double(value)
    }
}

impl From<Double> for f64 {
    fn from(value: Double) -> Self {
        value.0
    }
}

/// sRGB color value with optional alpha channel.
/// Format: #RRGGBB or #RRGGBBAA as defined in the 3MF Materials extension.
#[cfg_attr(feature = "speed-optimized-read", derive(Deserialize))]
#[cfg_attr(feature = "speed-optimized-read", serde(from = "String"))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    /// Red channel (0-255)
    pub r: u8,
    /// Green channel (0-255)
    pub g: u8,
    /// Blue channel (0-255)
    pub b: u8,
    /// Alpha channel (0-255), defaults to 255 (fully opaque)
    pub a: u8,
}

#[cfg(feature = "write")]
impl ToXml for Color {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _field: Option<Id<'_>>,
        serializer: &mut Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        let value = self.to_hex();
        serializer.write_str(&value)?;

        Ok(())
    }
}

#[cfg(feature = "memory-optimized-read")]
impl<'xml> FromXml<'xml> for Color {
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

        if let Some(value) = deserializer.take_str()? {
            if let Some(color_value) = Self::from_hex(&value) {
                *into = Some(color_value);
            } else {
                return Err(Error::MissingValue("Failed to parse f64 value of field {}"));
            }
        }

        Ok(())
    }

    type Accumulator = Option<Self>;
    const KIND: Kind = instant_xml::Kind::Scalar;
}

impl From<String> for Color {
    fn from(value: String) -> Self {
        Self::from_hex(&value).unwrap_or_default()
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

impl Color {
    /// Create a new opaque color with the given RGB values.
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Create a new color with RGBA values.
    pub fn with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// Parse a color from a hex string like "#RRGGBB" or "#RRGGBBAA".
    /// Returns None if the format is invalid.
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);

        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Self::new(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(Self::with_alpha(r, g, b, a))
            }
            _ => None,
        }
    }

    /// Format the color as a hex string.
    /// Always outputs 8 characters (RRGGBBAA format).
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
    }

    /// Format the color as a hex string, omitting alpha if fully opaque.
    pub fn to_hex_compact(&self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            self.to_hex()
        }
    }

    /// Convert to linear RGB values (0.0-1.0 range) for blending operations.
    /// Uses the inverse color component transfer function from the 3MF spec.
    pub fn to_linear(&self) -> (f64, f64, f64) {
        fn to_linear_channel(c: u8) -> f64 {
            let c_srgb = c as f64 / 255.0;
            if c_srgb <= 0.04045 {
                c_srgb / 12.92
            } else {
                ((c_srgb + 0.055) / 1.055).powf(2.4)
            }
        }
        (
            to_linear_channel(self.r),
            to_linear_channel(self.g),
            to_linear_channel(self.b),
        )
    }

    /// Create from linear RGB values (0.0-1.0 range).
    /// Uses the forward color component transfer function from the 3MF spec.
    pub fn from_linear(r: f64, g: f64, b: f64) -> Self {
        fn from_linear_channel(c: f64) -> u8 {
            let c_srgb = if c <= 0.0031308 {
                c * 12.92
            } else {
                (c.powf(1.0 / 2.4) * 1.055) - 0.055
            };
            (c_srgb * 255.0).round().clamp(0.0, 255.0) as u8
        }
        Self::new(
            from_linear_channel(r),
            from_linear_channel(g),
            from_linear_channel(b),
        )
    }
}

#[cfg(test)]
mod color_tests {
    use super::Color;

    #[test]
    fn color_new_defaults_to_opaque() {
        let c = Color::new(128, 64, 32);
        assert_eq!(c.r, 128);
        assert_eq!(c.g, 64);
        assert_eq!(c.b, 32);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn color_from_hex_6char() {
        let c = Color::from_hex("#FF8000").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 255);
    }

    #[test]
    fn color_from_hex_8char() {
        let c = Color::from_hex("#FF800080").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
        assert_eq!(c.a, 128);
    }

    #[test]
    fn color_from_hex_no_hash() {
        let c = Color::from_hex("FF8000").unwrap();
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn color_to_hex() {
        let c = Color::with_alpha(255, 128, 64, 200);
        assert_eq!(c.to_hex(), "#FF8040C8");
    }

    #[test]
    fn color_to_hex_compact_opaque() {
        let c = Color::new(255, 128, 64);
        assert_eq!(c.to_hex_compact(), "#FF8040");
    }

    #[test]
    fn color_to_hex_compact_with_alpha() {
        let c = Color::with_alpha(255, 128, 64, 200);
        assert_eq!(c.to_hex_compact(), "#FF8040C8");
    }

    #[test]
    fn color_roundtrip_hex() {
        let original = Color::with_alpha(100, 150, 200, 250);
        let hex = original.to_hex();
        let parsed = Color::from_hex(&hex).unwrap();
        assert_eq!(original, parsed);
    }

    #[test]
    fn color_linear_conversion() {
        // Test that linear conversion and back produces approximately the same value
        let original = Color::new(128, 64, 32);
        let linear = original.to_linear();
        let back = Color::from_linear(linear.0, linear.1, linear.2);
        // Allow for small rounding errors
        assert!((original.r as i16 - back.r as i16).abs() <= 1);
        assert!((original.g as i16 - back.g as i16).abs() <= 1);
        assert!((original.b as i16 - back.b as i16).abs() <= 1);
    }
}
