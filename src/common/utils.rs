use base64::Engine;
use chrono::{DateTime, Utc};
use image::ImageError;
use std::io::Cursor;
use uuid::Uuid;

/// Generate a new UUID v4 as a string
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Get current UTC timestamp
pub fn get_current_timestamp() -> DateTime<Utc> {
    Utc::now()
}

/// Format a duration in milliseconds to a human-readable string
pub fn format_duration_ms(duration_ms: u64) -> String {
    if duration_ms < 1000 {
        format!("{}ms", duration_ms)
    } else if duration_ms < 60000 {
        format!("{:.2}s", duration_ms as f64 / 1000.0)
    } else {
        let minutes = duration_ms / 60000;
        let seconds = (duration_ms % 60000) as f64 / 1000.0;
        format!("{}m {:.2}s", minutes, seconds)
    }
}

pub fn read_pngs_to_base64(paths: &[String]) -> Result<Vec<String>, ImageError> {
    let mut base64_images = Vec::new();
    for path in paths {
        let image = image::open(path)?;
        let mut buffer = Cursor::new(Vec::new());
        image.write_to(&mut buffer, image::ImageOutputFormat::Png)?;
        base64_images.push(base64::engine::general_purpose::STANDARD.encode(buffer.into_inner()));
    }
    Ok(base64_images)
}
