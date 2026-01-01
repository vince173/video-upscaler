//! Internationalization (i18n) module
//!
//! Manages language translations and user language preferences.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    English,
    Chinese,
}

impl Language {
    /// Get the language code for file naming
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Chinese => "zh",
        }
    }

    /// Parse from language code
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Language::English),
            "zh" => Some(Language::Chinese),
            _ => None,
        }
    }
}

/// Global translations cache
static TRANSLATIONS: OnceLock<Mutex<HashMap<Language, Value>>> = OnceLock::new();

/// Global current language preference
static CURRENT_LANGUAGE: OnceLock<Mutex<Language>> = OnceLock::new();

/// Get the translations directory
fn get_i18n_dir() -> PathBuf {
    // In development, look for i18n folder next to Cargo.toml
    if let Ok(exe_path) = std::env::current_exe() {
        let mut dir = exe_path.parent().unwrap_or_else(|| Path::new("."));
        // Go up from target/debug or target/release to project root
        if dir.ends_with("debug") || dir.ends_with("release") {
            dir = dir.parent().unwrap_or(dir);
            dir = dir.parent().unwrap_or(dir);
        }
        dir.join("i18n")
    } else {
        PathBuf::from("./i18n")
    }
}

/// Load translations for a specific language from JSON file
fn load_translations(lang: Language) -> Result<Value, String> {
    let i18n_dir = get_i18n_dir();
    let file_path = i18n_dir.join(format!("{}.json", lang.code()));

    fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read translation file {:?}: {}", file_path, e))
        .and_then(|content| {
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse translation file {:?}: {}", file_path, e))
        })
}

/// Get translations for a language, loading from file if not cached
fn get_translations(lang: Language) -> Result<Value, String> {
    // Ensure cache is initialized
    let cache = TRANSLATIONS.get_or_init(|| Mutex::new(HashMap::new()));

    // Check if already loaded
    {
        let cache_read = cache.lock().unwrap();
        if let Some(translations) = cache_read.get(&lang) {
            return Ok(translations.clone());
        }
    }

    // Load and cache
    let translations = load_translations(lang)?;
    {
        let mut cache_write = cache.lock().unwrap();
        cache_write.insert(lang, translations.clone());
    }

    Ok(translations)
}

/// Set the current language
pub fn set_language(lang: Language) -> Result<(), String> {
    // Pre-load translations to ensure they exist
    get_translations(lang)?;

    let current = CURRENT_LANGUAGE.get_or_init(|| Mutex::new(Language::Chinese));
    *current.lock().unwrap() = lang;
    Ok(())
}

/// Get the current language
pub fn get_language() -> Language {
    let current = CURRENT_LANGUAGE.get_or_init(|| Mutex::new(Language::Chinese));
    *current.lock().unwrap()
}

/// Get all translations for the current language (for frontend)
pub fn get_all_translations() -> Result<Value, String> {
    let lang = get_language();
    get_translations(lang)
}

/// Initialize i18n module with a saved language preference
pub fn init() {
    // Load saved preference or use default
    let lang = load_language_preference().unwrap_or(Language::Chinese);
    let _ = set_language(lang);
}

/// Load language preference from config file
fn load_language_preference() -> Option<Language> {
    // For now, use Chinese as default
    // In the future, this could read from a config file
    None
}

/// Save language preference to config file
pub fn save_language_preference(lang: Language) -> Result<(), String> {
    // For now, just store in memory
    // In the future, this could write to a config file
    set_language(lang)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_codes() {
        assert_eq!(Language::English.code(), "en");
        assert_eq!(Language::Chinese.code(), "zh");
    }

    #[test]
    fn test_language_from_code() {
        assert_eq!(Language::from_code("en"), Some(Language::English));
        assert_eq!(Language::from_code("zh"), Some(Language::Chinese));
        assert_eq!(Language::from_code("invalid"), None);
    }
}
