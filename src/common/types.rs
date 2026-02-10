use base64::Engine;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Base64Image {
    pub name: String,
    pub base64: String,
    pub format: ImageFormat,
    pub size: Option<u64>,
    pub metadata: Option<ImageMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageMetadata {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub created_at: Option<DateTime<Utc>>,
    pub file_size: Option<u64>,
    pub mime_type: Option<String>,
}

impl Base64Image {
    /// Create a new Base64Image with basic validation
    pub fn new(name: String, base64: String, format: ImageFormat) -> Result<Self, String> {
        if name.is_empty() {
            return Err("Image name cannot be empty".to_string());
        }

        if base64.is_empty() {
            return Err("Base64 data cannot be empty".to_string());
        }

        // Basic base64 validation (check if it's valid base64)
        if let Err(_) = base64::engine::general_purpose::STANDARD.decode(&base64) {
            return Err("Invalid base64 data".to_string());
        }

        Ok(Self {
            name,
            base64,
            format,
            size: None,
            metadata: None,
        })
    }

    /// Validate the base64 data
    pub fn validate_base64(&self) -> Result<(), String> {
        if self.base64.is_empty() {
            return Err("Base64 data is empty".to_string());
        }

        match base64::engine::general_purpose::STANDARD.decode(&self.base64) {
            Ok(_) => Ok(()),
            Err(_) => Err("Invalid base64 encoding".to_string()),
        }
    }

    /// Get the decoded size of the base64 data
    pub fn decoded_size(&self) -> Result<usize, String> {
        match base64::engine::general_purpose::STANDARD.decode(&self.base64) {
            Ok(decoded) => Ok(decoded.len()),
            Err(_) => Err("Failed to decode base64 data".to_string()),
        }
    }

    /// Check if the image has metadata
    pub fn has_metadata(&self) -> bool {
        self.metadata.is_some()
    }

    /// Get the MIME type for this image format
    pub fn mime_type(&self) -> &'static str {
        match self.format {
            ImageFormat::Png => "image/png",
            ImageFormat::WebP => "image/webp",
        }
    }

    /// Get dimensions as a tuple (width, height)
    pub fn dimensions(&self) -> Option<(u32, u32)> {
        self.metadata
            .as_ref()
            .and_then(|meta| match (meta.width, meta.height) {
                (Some(w), Some(h)) => Some((w, h)),
                _ => None,
            })
    }

    /// Set metadata
    pub fn set_metadata(&mut self, metadata: ImageMetadata) {
        self.metadata = Some(metadata);
    }

    /// Get metadata reference
    pub fn get_metadata(&self) -> Option<&ImageMetadata> {
        self.metadata.as_ref()
    }

    /// Get mutable metadata reference
    pub fn get_metadata_mut(&mut self) -> Option<&mut ImageMetadata> {
        self.metadata.as_mut()
    }
}

/// Builder pattern for Base64Image
#[derive(Debug, Default)]
pub struct Base64ImageBuilder {
    name: Option<String>,
    base64: Option<String>,
    format: Option<ImageFormat>,
    size: Option<u64>,
    metadata: ImageMetadata,
}

impl Base64ImageBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn base64(mut self, base64: String) -> Self {
        self.base64 = Some(base64);
        self
    }

    pub fn format(mut self, format: ImageFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn size(mut self, size: u64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn width(mut self, width: u32) -> Self {
        self.metadata.width = Some(width);
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.metadata.height = Some(height);
        self
    }

    pub fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.metadata.created_at = Some(created_at);
        self
    }

    pub fn file_size(mut self, file_size: u64) -> Self {
        self.metadata.file_size = Some(file_size);
        self
    }

    pub fn mime_type(mut self, mime_type: String) -> Self {
        self.metadata.mime_type = Some(mime_type);
        self
    }

    pub fn build(self) -> Result<Base64Image, String> {
        let name = self.name.ok_or("Name is required")?;
        let base64 = self.base64.ok_or("Base64 data is required")?;
        let format = self.format.ok_or("Format is required")?;

        let mut image = Base64Image::new(name, base64, format)?;
        image.size = self.size;
        image.metadata = Some(self.metadata);

        Ok(image)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageFormat {
    Png,
    WebP,
}

impl ImageFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "png" => Some(ImageFormat::Png),
            "webp" => Some(ImageFormat::WebP),
            _ => None,
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::WebP => "webp",
        }
    }

    pub fn to_image_format(&self) -> image::ImageFormat {
        match self {
            ImageFormat::Png => image::ImageFormat::Png,
            ImageFormat::WebP => image::ImageFormat::WebP,
        }
    }
}
