use crate::types::AiLibError;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Minimal file management helpers for multimodal flows.
/// - save_temp_file: write bytes to a temp file and return the path
/// - read_file: read bytes from a path
/// - remove_file: delete a file
/// - guess_mime_from_path: lightweight MIME guesser based on extension
/// - validate_file: validate file exists and is readable
/// - get_file_size: get file size in bytes
/// - create_temp_dir: create a temporary directory

/// Save bytes to a temporary file with a given prefix
pub fn save_temp_file(prefix: &str, bytes: &[u8]) -> io::Result<PathBuf> {
    let mut dir = std::env::temp_dir();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let filename = format!("{}-{}.bin", prefix, ts);
    dir.push(filename);
    fs::write(&dir, bytes)?;
    Ok(dir)
}

/// Read bytes from a file path
pub fn read_file(path: &Path) -> io::Result<Vec<u8>> {
    fs::read(path)
}

/// Remove a file from the filesystem
pub fn remove_file(path: &Path) -> io::Result<()> {
    fs::remove_file(path)
}

/// Guess MIME type from file extension
pub fn guess_mime_from_path(path: &Path) -> &'static str {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        match ext.to_ascii_lowercase().as_str() {
            "png" => "image/png",
            "jpg" => "image/jpeg",
            "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            "ogg" => "audio/ogg",
            "mp4" => "video/mp4",
            "avi" => "video/x-msvideo",
            "mov" => "video/quicktime",
            "pdf" => "application/pdf",
            "txt" => "text/plain",
            "md" => "text/markdown",
            "json" => "application/json",
            "xml" => "application/xml",
            "html" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "zip" => "application/zip",
            "tar" => "application/x-tar",
            "gz" => "application/gzip",
            _ => "application/octet-stream",
        }
    } else {
        "application/octet-stream"
    }
}

/// Validate that a file exists and is readable
pub fn validate_file(path: &Path) -> Result<(), AiLibError> {
    if !path.exists() {
        return Err(AiLibError::FileError(format!(
            "File does not exist: {}",
            path.display()
        )));
    }

    if !path.is_file() {
        return Err(AiLibError::FileError(format!(
            "Path is not a file: {}",
            path.display()
        )));
    }

    // Check if file is readable
    fs::metadata(path)
        .map_err(|e| AiLibError::FileError(format!("Cannot read file metadata: {}", e)))?;

    Ok(())
}

/// Get file size in bytes
pub fn get_file_size(path: &Path) -> Result<u64, AiLibError> {
    let metadata = fs::metadata(path)
        .map_err(|e| AiLibError::FileError(format!("Cannot read file metadata: {}", e)))?;
    Ok(metadata.len())
}

/// Create a temporary directory with a given prefix
pub fn create_temp_dir(prefix: &str) -> io::Result<PathBuf> {
    let mut dir = std::env::temp_dir();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dirname = format!("{}-{}", prefix, ts);
    dir.push(dirname);
    fs::create_dir(&dir)?;
    Ok(dir)
}

/// Check if a file is an image based on its extension
pub fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        matches!(
            ext.to_ascii_lowercase().as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tiff" | "svg"
        )
    } else {
        false
    }
}

/// Check if a file is an audio file based on its extension
pub fn is_audio_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        matches!(
            ext.to_ascii_lowercase().as_str(),
            "mp3" | "wav" | "ogg" | "flac" | "aac" | "m4a" | "wma"
        )
    } else {
        false
    }
}

/// Check if a file is a video file based on its extension
pub fn is_video_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        matches!(
            ext.to_ascii_lowercase().as_str(),
            "mp4" | "avi" | "mov" | "wmv" | "flv" | "mkv" | "webm" | "m4v"
        )
    } else {
        false
    }
}

/// Check if a file is a text file based on its extension
pub fn is_text_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        matches!(
            ext.to_ascii_lowercase().as_str(),
            "txt"
                | "md"
                | "json"
                | "xml"
                | "html"
                | "css"
                | "js"
                | "rs"
                | "py"
                | "java"
                | "cpp"
                | "c"
        )
    } else {
        false
    }
}

/// Get file extension as lowercase string
pub fn get_file_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase())
}

/// Check if file size is within acceptable limits
pub fn is_file_size_acceptable(path: &Path, max_size_mb: u64) -> Result<bool, AiLibError> {
    let size = get_file_size(path)?;
    let max_size_bytes = max_size_mb * 1024 * 1024;
    Ok(size <= max_size_bytes)
}
