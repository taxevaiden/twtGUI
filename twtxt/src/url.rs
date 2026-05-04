//! Module providing URL utilities, including media type detection.

use url::Url;

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

fn url_extension(url: &str) -> Option<String> {
    url.parse::<Url>()
        .ok()?
        .path_segments()?
        .next_back()?
        .rsplit('.')
        .next()
        .map(|e| e.to_lowercase())
}

pub fn is_media_url(url: &str) -> bool {
    let Some(ext) = url_extension(url) else {
        return false;
    };
    SKIP_EXTENSIONS.iter().any(|s| s.contains(&ext.as_str()))
}

pub fn is_image_url(url: &str) -> bool {
    let Some(ext) = url_extension(url) else {
        return false;
    };
    IMAGE_EXTENSIONS.contains(&ext.as_str())
}

pub fn is_video_url(url: &str) -> bool {
    let Some(ext) = url_extension(url) else {
        return false;
    };
    VIDEO_EXTENSIONS.contains(&ext.as_str())
}

pub fn is_audio_url(url: &str) -> bool {
    let Some(ext) = url_extension(url) else {
        return false;
    };
    AUDIO_EXTENSIONS.contains(&ext.as_str())
}

pub fn is_doc_url(url: &str) -> bool {
    let Some(ext) = url_extension(url) else {
        return false;
    };
    DOC_EXTENSIONS.contains(&ext.as_str())
}

pub fn is_text_url(url: &str) -> bool {
    let Some(ext) = url_extension(url) else {
        return false;
    };
    TEXT_EXTENSIONS.contains(&ext.as_str())
}

pub fn is_twtxt_url(url: &str) -> bool {
    url.contains("twtxt.txt") || url.contains("twtxt")
}
