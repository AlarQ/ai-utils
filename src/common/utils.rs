use base64::Engine;
use futures::future::try_join_all;
use image::GenericImageView;
use std::io::Cursor;

use super::{
    errors::CommonError,
    types::{Base64Image, ImageFormat, ImageMetadata},
};

// --- ASYNC VERSIONS ---
use tokio::{fs as async_fs, io::AsyncReadExt};

/// Generic async function to convert a single image to base64
pub async fn read_image_to_base64(path: &str, format: ImageFormat) -> Result<String, CommonError> {
    let mut file = async_fs::File::open(path)
        .await
        .map_err(|e| CommonError::FileRead(format!("Failed to open image at {}: {}", path, e)))?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .await
        .map_err(|e| CommonError::FileRead(format!("Failed to read image at {}: {}", path, e)))?;

    // Process image in blocking thread to avoid blocking async runtime
    let format_clone = format;

    let base64 = tokio::task::spawn_blocking(move || {
        let image = image::load_from_memory(&buf).map_err(|e| CommonError::Image(e))?;

        let mut buffer = Cursor::new(Vec::new());
        image
            .write_to(&mut buffer, format_clone.to_image_format())
            .map_err(|e| CommonError::Image(e))?;

        let bytes = buffer.into_inner();
        Ok::<String, CommonError>(base64::engine::general_purpose::STANDARD.encode(bytes))
    })
    .await
    .map_err(|e| CommonError::FileRead(format!("Task join error: {}", e)))??;

    Ok(base64)
}

/// Process a single image file with metadata extraction
async fn process_image_file(path: String, format: ImageFormat) -> Result<Base64Image, CommonError> {
    let base64 = read_image_to_base64(&path, format).await?;

    let name = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| CommonError::InvalidPath(format!("Invalid filename: {}", path)))?
        .to_string();

    // Extract metadata in parallel
    let metadata_future = tokio::task::spawn_blocking({
        let path_clone = path.clone();
        move || {
            let image = image::open(&path_clone).ok();
            if let Some(img) = image {
                let (width, height) = img.dimensions();
                Some((width, height))
            } else {
                None
            }
        }
    });

    let mut base64_image = Base64Image::new(name, base64, format)
        .map_err(|e| CommonError::FileRead(format!("Failed to create Base64Image: {}", e)))?;

    // Add metadata if available
    if let Ok(Some((width, height))) = metadata_future.await {
        let mut image_metadata = ImageMetadata::default();
        image_metadata.width = Some(width);
        image_metadata.height = Some(height);
        base64_image.set_metadata(image_metadata);
    }

    Ok(base64_image)
}

/// Generic async function to convert all images of a specific format in a directory to base64 with parallel processing
pub async fn read_images_to_base64(
    directory: &str,
    format: ImageFormat,
) -> Result<Vec<Base64Image>, CommonError> {
    // Collect all valid file paths first
    let mut valid_paths = Vec::new();
    let mut entries = async_fs::read_dir(directory).await.map_err(|e| {
        CommonError::DirectoryRead(format!("Failed to read directory {}: {}", directory, e))
    })?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| CommonError::DirectoryRead(format!("Failed to read directory entry: {}", e)))?
    {
        let path = entry.path();

        if path.is_file() {
            let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("");

            if let Some(file_format) = ImageFormat::from_extension(extension) {
                if file_format == format {
                    let path_str = path
                        .to_str()
                        .ok_or_else(|| {
                            CommonError::InvalidPath(format!("Invalid path: {:?}", path))
                        })?
                        .to_string();
                    valid_paths.push(path_str);
                }
            }
        }
    }

    if valid_paths.is_empty() {
        return Err(CommonError::NoValidFiles(format!(
            "No {} files found in directory: {}",
            format.extension(),
            directory
        )));
    }

    // Process files in parallel with configurable concurrency
    let max_concurrent = std::env::var("IMAGE_PROCESSING_CONCURRENCY")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(4); // Default to 4 concurrent tasks

    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_concurrent));

    let futures: Vec<_> = valid_paths
        .into_iter()
        .map(|path| {
            let semaphore = semaphore.clone();
            let format_clone = format;

            async move {
                let _permit = semaphore
                    .acquire()
                    .await
                    .map_err(|e| CommonError::FileRead(format!("Semaphore error: {}", e)))?;
                process_image_file(path, format_clone).await
            }
        })
        .collect();

    // Wait for all tasks to complete
    let results = try_join_all(futures).await?;

    Ok(results)
}

/// Convenience function for PNG images
pub async fn read_png_to_base64(path: &str) -> Result<String, CommonError> {
    read_image_to_base64(path, ImageFormat::Png).await
}

/// Convenience function for PNG images in directory
pub async fn read_pngs_to_base64(directory: &str) -> Result<Vec<Base64Image>, CommonError> {
    read_images_to_base64(directory, ImageFormat::Png).await
}

/// Convenience function for WebP images
pub async fn read_webp_to_base64(path: &str) -> Result<String, CommonError> {
    read_image_to_base64(path, ImageFormat::WebP).await
}

/// Convenience function for WebP images in directory
pub async fn read_webps_to_base64(directory: &str) -> Result<Vec<Base64Image>, CommonError> {
    read_images_to_base64(directory, ImageFormat::WebP).await
}
