//! Shared utilities used throughout the application.

pub mod download;
pub mod hash;
pub mod paths;
pub mod styling;

use reqwest::Url;

const IMAGE_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "gif", "webp", "svg", "bmp", "ico", "tiff",
];

const VIDEO_EXTENSIONS: &[&str] = &["mp4", "webm", "mov", "avi", "mkv"];

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "ogg", "wav", "flac"];

const DOC_EXTENSIONS: &[&str] = &["pdf", "zip", "tar", "gz", "rar", "7z"];

const TEXT_EXTENSIONS: &[&str] = &["txt", "json", "xml", "csv"];

const SKIP_EXTENSIONS: &[&[&str]] = &[
    IMAGE_EXTENSIONS,
    VIDEO_EXTENSIONS,
    AUDIO_EXTENSIONS,
    DOC_EXTENSIONS,
    TEXT_EXTENSIONS,
];

pub fn is_file_url(url: &str) -> bool {
    let Ok(parsed) = url.parse::<Url>() else {
        return false;
    };
    let ext = parsed
        .path_segments()
        .and_then(|mut s| s.next_back())
        .and_then(|seg| seg.rsplit('.').next())
        .unwrap_or("")
        .to_lowercase();
    SKIP_EXTENSIONS.iter().any(|s| s.contains(&ext.as_str()))
}

pub fn is_image_url(url: &str) -> bool {
    let Ok(parsed) = url.parse::<Url>() else {
        return false;
    };
    let ext = parsed
        .path_segments()
        .and_then(|mut s| s.next_back())
        .and_then(|seg| seg.rsplit('.').next())
        .unwrap_or("")
        .to_lowercase();
    IMAGE_EXTENSIONS.contains(&ext.as_str())
}
