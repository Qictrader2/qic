use crate::error::{Result, TwolebotError};
use std::fs;
use std::path::{Path, PathBuf};

/// MIME type mapping for Telegram media types
/// Note: This provides default MIME types when Telegram doesn't include one
pub fn mime_for_telegram_media(media_type: &str) -> &'static str {
    match media_type {
        "voice" => "audio/ogg",
        "audio" => "audio/mpeg",
        "video" => "video/mp4",
        "video_note" => "video/mp4",
        "photo" => "image/jpeg",
        "document" => "application/octet-stream",
        "sticker" => "image/webp",
        "animation" => "video/mp4", // Telegram converts GIFs to MP4
        _ => "application/octet-stream",
    }
}

/// Manages media file storage per chat
pub struct MediaStore {
    base_dir: PathBuf,
}

impl MediaStore {
    pub fn new(base_dir: impl AsRef<Path>) -> Result<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        fs::create_dir_all(&base_dir).map_err(TwolebotError::from)?;
        Ok(Self { base_dir })
    }

    /// Validate that a path component doesn't contain traversal attacks
    fn validate_path_component(component: &str) -> Result<()> {
        // Reject empty, ".", "..", or anything with path separators
        if component.is_empty()
            || component == "."
            || component == ".."
            || component.contains('/')
            || component.contains('\\')
            || component.contains('\0')
        {
            return Err(TwolebotError::other(format!(
                "Invalid path component: '{}'",
                component
            )));
        }
        Ok(())
    }

    fn chat_dir(&self, chat_id: &str) -> PathBuf {
        self.base_dir.join(chat_id)
    }

    /// Get the path where a media file should be stored (with validation)
    pub fn media_path(&self, chat_id: &str, filename: &str) -> PathBuf {
        self.chat_dir(chat_id).join(filename)
    }

    /// Get a safe, validated media path
    /// Returns an error if chat_id or filename contain path traversal attempts
    pub fn safe_media_path(&self, chat_id: &str, filename: &str) -> Result<PathBuf> {
        Self::validate_path_component(chat_id)?;
        Self::validate_path_component(filename)?;

        let path = self.media_path(chat_id, filename);

        // Double-check: canonicalize and verify it's under base_dir
        // Note: file may not exist yet, so we check the parent directory
        let chat_dir = self.chat_dir(chat_id);
        if chat_dir.exists() {
            let canonical_base = self.base_dir.canonicalize()?;
            let canonical_chat = chat_dir.canonicalize()?;
            if !canonical_chat.starts_with(&canonical_base) {
                return Err(TwolebotError::other("Path traversal detected"));
            }
        }

        Ok(path)
    }

    /// Store media data and return the path
    pub fn store(&self, chat_id: &str, filename: &str, data: &[u8]) -> Result<PathBuf> {
        let path = self.safe_media_path(chat_id, filename)?;
        let chat_dir = path.parent()
            .ok_or_else(|| TwolebotError::other("Invalid media path: no parent directory"))?;
        fs::create_dir_all(&chat_dir)?;

        fs::write(&path, data)?;
        Ok(path)
    }

    /// Read media data
    pub fn read(&self, chat_id: &str, filename: &str) -> Result<Vec<u8>> {
        let path = self.media_path(chat_id, filename);
        let data = fs::read(&path)?;
        Ok(data)
    }

    /// Check if a media file exists
    pub fn exists(&self, chat_id: &str, filename: &str) -> bool {
        self.media_path(chat_id, filename).exists()
    }

    /// Delete a media file
    pub fn delete(&self, chat_id: &str, filename: &str) -> Result<bool> {
        let path = self.media_path(chat_id, filename);
        match fs::remove_file(&path) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    /// List all media files for a chat
    pub fn list(&self, chat_id: &str) -> Result<Vec<String>> {
        let chat_dir = self.chat_dir(chat_id);
        if !chat_dir.exists() {
            return Ok(Vec::new());
        }

        let mut files = Vec::new();
        for entry in fs::read_dir(&chat_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(name) = entry.file_name().to_str() {
                    files.push(name.to_string());
                }
            }
        }

        Ok(files)
    }

    /// Delete all media files for a chat (removes the entire chat directory)
    pub fn delete_chat(&self, chat_id: &str) -> Result<()> {
        Self::validate_path_component(chat_id)?;
        let chat_dir = self.chat_dir(chat_id);
        match fs::remove_dir_all(&chat_dir) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// Get total size of media for a chat
    pub fn chat_size(&self, chat_id: &str) -> Result<u64> {
        let chat_dir = self.chat_dir(chat_id);
        if !chat_dir.exists() {
            return Ok(0);
        }

        let mut total = 0;
        for entry in fs::read_dir(&chat_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                total += entry.metadata()?.len();
            }
        }

        Ok(total)
    }

}

/// Get file extension from MIME type
pub fn extension_for_mime(mime: &str) -> &'static str {
    match mime {
        // Audio
        "audio/ogg" | "audio/opus" => "ogg",
        "audio/mpeg" | "audio/mp3" => "mp3",
        "audio/wav" | "audio/wave" | "audio/x-wav" => "wav",
        "audio/mp4" | "audio/m4a" | "audio/x-m4a" => "m4a",
        "audio/flac" | "audio/x-flac" => "flac",
        "audio/aac" => "aac",
        // Video
        "video/mp4" => "mp4",
        "video/webm" => "webm",
        "video/quicktime" => "mov",
        "video/x-msvideo" => "avi",
        "video/x-matroska" => "mkv",
        // Images
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/bmp" => "bmp",
        "image/tiff" => "tiff",
        // Documents
        "application/pdf" => "pdf",
        "text/plain" => "txt",
        "application/json" => "json",
        "application/xml" | "text/xml" => "xml",
        "application/zip" => "zip",
        // Telegram specific
        "application/x-tgsticker" => "tgs",
        _ => "bin",
    }
}

/// Get MIME type from file extension
pub fn mime_for_extension(ext: &str) -> &'static str {
    match ext.to_lowercase().as_str() {
        // Audio
        "ogg" | "oga" | "opus" => "audio/ogg",
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "m4a" => "audio/mp4",
        "flac" => "audio/flac",
        "aac" => "audio/aac",
        // Video
        "mp4" | "m4v" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "mkv" => "video/x-matroska",
        // Images
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        "tiff" | "tif" => "image/tiff",
        // Documents
        "pdf" => "application/pdf",
        "txt" => "text/plain",
        "json" => "application/json",
        "xml" => "application/xml",
        "zip" => "application/zip",
        // Telegram specific
        "tgs" => "application/x-tgsticker",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_media_store_basic() {
        let dir = tempdir().unwrap();
        let store = MediaStore::new(dir.path()).unwrap();

        // Store media
        let data = b"fake image data";
        let path = store.store("chat-123", "photo.jpg", data).unwrap();

        assert!(path.exists());
        assert!(store.exists("chat-123", "photo.jpg"));

        // Read media
        let read_data = store.read("chat-123", "photo.jpg").unwrap();
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_media_store_list() {
        let dir = tempdir().unwrap();
        let store = MediaStore::new(dir.path()).unwrap();

        store.store("chat-123", "photo1.jpg", b"data1").unwrap();
        store.store("chat-123", "photo2.jpg", b"data2").unwrap();
        store.store("chat-123", "voice.ogg", b"data3").unwrap();

        let files = store.list("chat-123").unwrap();
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_media_store_size() {
        let dir = tempdir().unwrap();
        let store = MediaStore::new(dir.path()).unwrap();

        store.store("chat-123", "file1.bin", &[0u8; 100]).unwrap();
        store.store("chat-123", "file2.bin", &[0u8; 200]).unwrap();

        let size = store.chat_size("chat-123").unwrap();
        assert_eq!(size, 300);
    }

    #[test]
    fn test_mime_mapping() {
        assert_eq!(mime_for_telegram_media("voice"), "audio/ogg");
        assert_eq!(mime_for_telegram_media("video"), "video/mp4");
        assert_eq!(mime_for_telegram_media("photo"), "image/jpeg");

        assert_eq!(extension_for_mime("audio/ogg"), "ogg");
        assert_eq!(extension_for_mime("image/jpeg"), "jpg");

        assert_eq!(mime_for_extension("ogg"), "audio/ogg");
        assert_eq!(mime_for_extension("jpg"), "image/jpeg");
    }

    #[test]
    fn test_path_traversal_protection() {
        let dir = tempdir().unwrap();
        let store = MediaStore::new(dir.path()).unwrap();

        // These should all be rejected
        assert!(store.safe_media_path("..", "file.jpg").is_err());
        assert!(store.safe_media_path("chat", "..").is_err());
        assert!(store.safe_media_path("chat", "../etc/passwd").is_err());
        assert!(store.safe_media_path("../etc", "passwd").is_err());
        assert!(store.safe_media_path("chat", "foo/bar").is_err());
        assert!(store.safe_media_path("chat/subdir", "file.jpg").is_err());
        assert!(store.safe_media_path(".", "file.jpg").is_err());
        assert!(store.safe_media_path("chat", ".").is_err());
        assert!(store.safe_media_path("", "file.jpg").is_err());
        assert!(store.safe_media_path("chat", "").is_err());

        // These should be allowed
        assert!(store.safe_media_path("chat-123", "photo.jpg").is_ok());
        assert!(store.safe_media_path("-12345", "voice_001.ogg").is_ok());
        assert!(store.safe_media_path("123456789", "file.bin").is_ok());
    }
}
