use base64::Engine;
use std::fs;
use std::io::Cursor;

use super::errors::CommonError;
use super::types::Base64Image;

pub fn read_png_to_base64(path: &str) -> Result<String, CommonError> {
    let image = image::open(path)
        .map_err(|e| CommonError::FileRead(format!("Failed to open image at {}: {}", path, e)))?;

    let mut buffer = Cursor::new(Vec::new());
    image
        .write_to(&mut buffer, image::ImageOutputFormat::Png)
        .map_err(|e| CommonError::Image(e))?;

    let bytes = buffer.into_inner();
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

pub fn read_pngs_to_base64(directory: &str) -> Result<Vec<Base64Image>, CommonError> {
    let mut base64_images = Vec::new();

    let entries = fs::read_dir(directory).map_err(|e| {
        CommonError::DirectoryRead(format!("Failed to read directory {}: {}", directory, e))
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            CommonError::DirectoryRead(format!("Failed to read directory entry: {}", e))
        })?;

        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "png") {
            let path_str = path
                .to_str()
                .ok_or_else(|| CommonError::InvalidPath(format!("Invalid path: {:?}", path)))?;

            let base64 = read_png_to_base64(path_str)?;

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| CommonError::InvalidPath(format!("Invalid filename: {:?}", path)))?
                .to_string();

            base64_images.push(Base64Image { name, base64 });
        }
    }

    if base64_images.is_empty() {
        return Err(CommonError::NoValidFiles(format!(
            "No PNG files found in directory: {}",
            directory
        )));
    }

    Ok(base64_images)
}

pub fn read_webp_to_base64(path: &str) -> Result<String, CommonError> {
    let image = image::open(path)
        .map_err(|e| CommonError::FileRead(format!("Failed to open image at {}: {}", path, e)))?;

    let mut buffer = Cursor::new(Vec::new());
    image
        .write_to(&mut buffer, image::ImageOutputFormat::WebP)
        .map_err(|e| CommonError::Image(e))?;

    let bytes = buffer.into_inner();
    Ok(base64::engine::general_purpose::STANDARD.encode(bytes))
}

pub fn read_webps_to_base64(directory: &str) -> Result<Vec<Base64Image>, CommonError> {
    let mut base64_images = Vec::new();

    let entries = fs::read_dir(directory).map_err(|e| {
        CommonError::DirectoryRead(format!("Failed to read directory {}: {}", directory, e))
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            CommonError::DirectoryRead(format!("Failed to read directory entry: {}", e))
        })?;

        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "webp") {
            let path_str = path
                .to_str()
                .ok_or_else(|| CommonError::InvalidPath(format!("Invalid path: {:?}", path)))?;

            let base64 = read_webp_to_base64(path_str)?;

            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| CommonError::InvalidPath(format!("Invalid filename: {:?}", path)))?
                .to_string();

            base64_images.push(Base64Image { name, base64 });
        }
    }

    if base64_images.is_empty() {
        return Err(CommonError::NoValidFiles(format!(
            "No WebP files found in directory: {}",
            directory
        )));
    }

    Ok(base64_images)
}
