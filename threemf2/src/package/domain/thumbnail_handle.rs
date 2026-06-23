//! Thumbnail image handling for 3MF packages.

use crate::model::StrResource;

/// Image format for thumbnails in a 3MF package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageFormat {
    /// PNG image format.
    Png,
    /// JPEG image format.
    Jpeg,
    /// Unknown image format with the extension.
    Unknown(StrResource),
}

impl ImageFormat {
    /// Creates an ImageFormat from a file extension.
    pub fn from_ext(ext: &str) -> Self {
        match ext.to_lowercase().as_ref() {
            "png" => Self::Png,
            "jpg" | "jpeg" => Self::Jpeg,
            _ => Self::Unknown(ext.into()),
        }
    }

    /// Returns the file extension as a string.
    pub fn as_str(&self) -> &str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpeg",
            ImageFormat::Unknown(ext) => ext,
        }
    }
}

/// Handle for a thumbnail image within a 3MF package.
#[derive(Debug, Clone, PartialEq)]
pub struct ThumbnailHandle {
    /// Raw image data.
    pub data: Vec<u8>,
    /// Image format of the thumbnail.
    pub format: ImageFormat,
}

#[cfg(test)]
mod tests {
    use crate::package::domain::thumbnail_handle::ImageFormat;

    #[test]
    fn from_ext_test() {
        let png = ImageFormat::from_ext("PNG");
        let jpg = ImageFormat::from_ext("JPG");
        let jpeg = ImageFormat::from_ext("JPEG");
        let unknown = ImageFormat::from_ext("tiff");

        assert_eq!(png, ImageFormat::Png);
        assert_eq!(jpg, ImageFormat::Jpeg);
        assert_eq!(jpeg, ImageFormat::Jpeg);
        assert_eq!(unknown, ImageFormat::Unknown("tiff".into()));
    }
}
