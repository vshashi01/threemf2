use crate::model::StrResource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Unknown(StrResource),
}

impl ImageFormat {
    pub fn from_ext(ext: &str) -> Self {
        match ext.to_lowercase().as_ref() {
            "png" => Self::Png,
            "jpg" | "jpeg" => Self::Jpeg,
            _ => Self::Unknown(ext.into()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpeg",
            ImageFormat::Unknown(ext) => ext,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThumbnailHandle {
    pub data: Vec<u8>,
    pub format: ImageFormat,
}

#[cfg(test)]
mod tests {
    use crate::io::thumbnail_handle::ImageFormat;

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
