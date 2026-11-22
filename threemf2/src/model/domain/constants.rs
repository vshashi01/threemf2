#[cfg(feature = "read")]
pub const MAX_VERTEX_BUFFER: usize = 1000;
#[cfg(feature = "read")]
pub const MAX_TRIANGLE_BUFFER: usize = 2000;

/// Parses a numeric XML attribute value strictly.
///
/// Leading/trailing ASCII whitespace is tolerated (real-world files contain
/// e.g. `p1="1 "`), anything else that is not a valid number is an error
/// instead of silently becoming a default value.
#[cfg(feature = "read")]
#[inline]
pub(crate) fn parse_num_attr<T: lexical_core::FromLexical>(
    value: &str,
    attr: &'static str,
) -> Result<T, instant_xml::Error> {
    lexical_core::parse(value.trim_ascii().as_bytes()).map_err(|_| {
        instant_xml::Error::UnexpectedValue(format!(
            "invalid number {value:?} for attribute {attr:?}"
        ))
    })
}
