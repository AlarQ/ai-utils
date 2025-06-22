use base64::Engine;
use image::ImageError;
use std::fs;
use std::io::Cursor;

use super::types::Base64Image;

pub fn read_png_to_base64(path: &str) -> Result<String, ImageError> {
    let image = image::open(path)?;
    let mut buffer = Cursor::new(Vec::new());
    image.write_to(&mut buffer, image::ImageOutputFormat::Png)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(buffer.into_inner()))
}

pub fn read_pngs_to_base64(directory: &str) -> Result<Vec<Base64Image>, ImageError> {
    let mut base64_images = Vec::new();
    let entries = fs::read_dir(directory).map_err(|_| {
        ImageError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to read directory",
        ))
    })?;
    for entry in entries {
        let entry = entry.map_err(|_| {
            ImageError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to read entry",
            ))
        })?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "png") {
            let base64 = read_png_to_base64(path.to_str().unwrap())?;
            base64_images.push(Base64Image {
                name: path.file_name().unwrap().to_str().unwrap().to_string(),
                base64,
            });
        }
    }
    Ok(base64_images)
}

pub fn read_webp_to_base64(path: &str) -> Result<String, ImageError> {
    let image = image::open(path)?;
    let mut buffer = Cursor::new(Vec::new());
    image.write_to(&mut buffer, image::ImageOutputFormat::WebP)?;
    Ok(base64::engine::general_purpose::STANDARD.encode(buffer.into_inner()))
}

pub fn read_webps_to_base64(directory: &str) -> Result<Vec<Base64Image>, ImageError> {
    let mut base64_images = Vec::new();
    let entries = fs::read_dir(directory).map_err(|_| {
        ImageError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to read directory",
        ))
    })?;
    for entry in entries {
        let entry = entry.map_err(|_| {
            ImageError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to read entry",
            ))
        })?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "webp") {
            let base64 = read_webp_to_base64(path.to_str().unwrap())?;
            base64_images.push(Base64Image {
                name: path.file_name().unwrap().to_str().unwrap().to_string(),
                base64,
            });
        }
    }
    Ok(base64_images)
}
