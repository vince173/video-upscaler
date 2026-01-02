//! Security utilities for validating and sanitizing user inputs

use crate::error::{UpscalerError, Result};
use std::path::{Path, PathBuf};

/// Validate that a file path is safe to access
///
/// Ensures:
/// - Path doesn't escape allowed directories via .. sequences
/// - Path is within user-accessible directories (not system directories)
/// - Path doesn't contain suspicious shell metacharacters
///
/// Note: This requires the file to exist (uses canonicalize).
pub fn validate_file_path(path: &Path) -> Result<PathBuf> {
    // Get the canonical form to resolve any .. sequences
    let canonical = path
        .canonicalize()
        .map_err(|e| UpscalerError::ConfigError(format!("Invalid path: {}", e)))?;

    let path_str = canonical.to_string_lossy();

    // Block obvious system directories on Windows
    #[cfg(target_os = "windows")]
    {
        let blocked_prefixes = [
            "C:\\Windows",
            "C:\\Program Files",
            "C:\\Program Files (x86)",
            "C:\\ProgramData",
            "\\Windows",
            "\\Program Files",
            "\\Program Files (x86)",
            "\\ProgramData",
        ];

        for blocked in &blocked_prefixes {
            if path_str.to_lowercase().starts_with(&blocked.to_lowercase()) {
                return Err(UpscalerError::ConfigError(format!(
                    "Access to system directory blocked: {}",
                    blocked
                )));
            }
        }
    }

    // Block obvious system directories on Unix-like systems
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let blocked_prefixes = ["/usr", "/bin", "/sbin", "/etc", "/sys", "/proc", "/boot"];

        for blocked in &blocked_prefixes {
            if path_str.starts_with(blocked) {
                return Err(UpscalerError::ConfigError(format!(
                    "Access to system directory blocked: {}",
                    blocked
                )));
            }
        }
    }

    // Check for shell metacharacters in the path that could indicate injection attempts
    let path_chars = path_str.chars().collect::<Vec<_>>();
    let dangerous_chars = ['|', '&', ';', '$', '`', '\n', '\r', '\t'];

    for &dangerous in &dangerous_chars {
        if path_chars.contains(&dangerous) {
            return Err(UpscalerError::ConfigError(format!(
                "Path contains invalid character: {}",
                dangerous
            )));
        }
    }

    Ok(canonical)
}

/// Validate that an output file path is safe to write to
///
/// Ensures:
/// - Path doesn't escape allowed directories via .. sequences
/// - Parent directory exists and is within user-accessible directories
/// - Path doesn't contain suspicious shell metacharacters
///
/// Note: This is for output files that don't exist yet.
pub fn validate_output_path(path: &Path) -> Result<PathBuf> {
    // For output files, validate the parent directory exists
    let parent = path
        .parent()
        .ok_or_else(|| UpscalerError::ConfigError("Path has no parent directory".to_string()))?;

    // Get canonical form of parent directory
    let canonical_parent = parent
        .canonicalize()
        .map_err(|e| UpscalerError::ConfigError(format!("Invalid parent directory: {}", e)))?;

    // Build the full output path from canonical parent + file name
    let file_name = path
        .file_name()
        .ok_or_else(|| UpscalerError::ConfigError("Path has no file name".to_string()))?;

    let canonical_output = canonical_parent.join(file_name);

    let path_str = canonical_parent.to_string_lossy();

    // Block obvious system directories on Windows
    #[cfg(target_os = "windows")]
    {
        let blocked_prefixes = [
            "C:\\Windows",
            "C:\\Program Files",
            "C:\\Program Files (x86)",
            "C:\\ProgramData",
            "\\Windows",
            "\\Program Files",
            "\\Program Files (x86)",
            "\\ProgramData",
        ];

        for blocked in &blocked_prefixes {
            if path_str.to_lowercase().starts_with(&blocked.to_lowercase()) {
                return Err(UpscalerError::ConfigError(format!(
                    "Access to system directory blocked: {}",
                    blocked
                )));
            }
        }
    }

    // Block obvious system directories on Unix-like systems
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let blocked_prefixes = ["/usr", "/bin", "/sbin", "/etc", "/sys", "/proc", "/boot"];

        for blocked in &blocked_prefixes {
            if path_str.starts_with(blocked) {
                return Err(UpscalerError::ConfigError(format!(
                    "Access to system directory blocked: {}",
                    blocked
                )));
            }
        }
    }

    // Check for shell metacharacters in the full output path that could indicate injection attempts
    let full_path_str = canonical_output.to_string_lossy();
    let path_chars = full_path_str.chars().collect::<Vec<_>>();
    let dangerous_chars = ['|', '&', ';', '$', '`', '\n', '\r', '\t'];

    for &dangerous in &dangerous_chars {
        if path_chars.contains(&dangerous) {
            return Err(UpscalerError::ConfigError(format!(
                "Path contains invalid character: {}",
                dangerous
            )));
        }
    }

    Ok(canonical_output)
}

/// Validate that a path is safe for deletion
///
/// Only allows deletion of files in temp directories, Downloads, or user-selected output locations
/// Note: This function handles files that may not exist yet (partial files during cancellation)
pub fn validate_deletion_path(path: &Path) -> Result<PathBuf> {
    // Try to canonicalize the path directly first
    let canonical = if path.exists() {
        let c = path.canonicalize()
            .map_err(|e| UpscalerError::ConfigError(format!("Invalid path: {}", e)))?;
        c
    } else {
        // File doesn't exist yet - validate the parent directory instead
        let parent = path.parent()
            .ok_or_else(|| UpscalerError::ConfigError("Path has no parent directory".to_string()))?;

        let canonical_parent = parent.canonicalize()
            .map_err(|e| UpscalerError::ConfigError(format!("Invalid parent directory: {}", e)))?;

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| UpscalerError::ConfigError("Invalid file name".to_string()))?;

        // Validate parent directory is within allowed locations
        let parent_str = canonical_parent.to_string_lossy();

        // Get temp directory
        let temp_dir = std::env::temp_dir();
        let temp_str = temp_dir.to_string_lossy().to_lowercase();

        // Get user's home directory
        let home_dir = dirs::home_dir();

        let allowed = if let Some(home) = home_dir {
            let home_str = home.to_string_lossy().to_lowercase();
            parent_str.to_lowercase().starts_with(&home_str)
                || parent_str.to_lowercase().starts_with(&temp_str)
        } else {
            parent_str.to_lowercase().starts_with(&temp_str)
        };

        if !allowed {
            return Err(UpscalerError::ConfigError(
                "File deletion not allowed from this location".to_string(),
            ));
        }

        // Return the full path (parent + filename) for the actual deletion attempt
        return Ok(canonical_parent.join(file_name));
    };

    // File exists - validate it's in an allowed location
    let path_str = canonical.to_string_lossy();

    // Strip Windows extended-length path prefix (\\?\) for comparison
    let path_str_for_comparison = if path_str.starts_with("\\\\?\\") {
        path_str.get(4..).unwrap_or(&path_str)  // Safe fallback if slice is too short
    } else {
        &path_str
    };

    // Get temp directory
    let temp_dir = std::env::temp_dir();
    let temp_str = temp_dir.to_string_lossy().to_lowercase();

    // Get user's home directory
    let home_dir = dirs::home_dir();

    let allowed = if let Some(home) = home_dir {
        let home_str = home.to_string_lossy().to_lowercase();
        path_str_for_comparison.to_lowercase().starts_with(&home_str)
            || path_str_for_comparison.to_lowercase().starts_with(&temp_str)
    } else {
        path_str_for_comparison.to_lowercase().starts_with(&temp_str)
    };

    if !allowed {
        return Err(UpscalerError::ConfigError(
            "File deletion not allowed from this location".to_string(),
        ));
    }

    Ok(canonical)
}

/// Validate that a path is a video file with allowed extension
pub fn validate_video_file(path: &Path) -> Result<PathBuf> {
    let canonical = validate_file_path(path)?;

    // Check file exists
    if !canonical.exists() {
        return Err(UpscalerError::ConfigError("File does not exist".to_string()));
    }

    // Check it's actually a file (not a directory)
    if !canonical.is_file() {
        return Err(UpscalerError::ConfigError(
            "Path is not a file".to_string(),
        ));
    }

    // Check file extension
    let allowed_extensions = ["mp4", "avi", "mov", "mkv", "webm", "flv", "wmv", "m4v"];

    let extension = canonical
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if !allowed_extensions.contains(&extension.as_str()) {
        return Err(UpscalerError::ConfigError(format!(
            "Invalid file extension: {}",
            extension
        )));
    }

    Ok(canonical)
}

/// Sanitize a path string for safe use in shell commands
///
/// On Windows: properly quotes paths for use with explorer.exe
/// On Unix: escapes special characters
pub fn sanitize_path_for_shell(path: &Path) -> String {
    #[cfg(target_os = "windows")]
    {
        // On Windows, wrap in quotes and escape existing quotes
        let path_str = path.to_string_lossy().replace('\"', "\"\"");
        format!("\"{}\"", path_str)
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        // On Unix, use shell escaping with single quotes
        let path_str = path.to_string_lossy();
        let escaped = path_str.replace('\'', "'\\''");
        format!("'{}'", escaped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_normal_file() {
        // This would need a real file to test, so we'll skip in unit tests
        // Integration tests should cover this
    }

    #[test]
    fn test_sanitize_windows_path() {
        #[cfg(target_os = "windows")]
        {
            let path = PathBuf::from(r"C:\Users\test\video.mp4");
            let sanitized = sanitize_path_for_shell(&path);
            assert!(sanitized.starts_with('"'));
            assert!(sanitized.ends_with('"'));
        }
    }

    #[test]
    fn test_sanitize_unix_path() {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            let path = PathBuf::from("/home/user/video.mp4");
            let sanitized = sanitize_path_for_shell(&path);
            assert!(sanitized.starts_with('\''));
            assert!(sanitized.ends_with('\''));
        }
    }
}
