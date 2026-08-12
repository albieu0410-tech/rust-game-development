use std::fs;
use std::path::PathBuf;

use thiserror::Error;

use crate::Profile;

#[derive(Debug, Error)]
pub enum SaveError {
    #[error("save profile was not found")]
    NotFound,
    #[error("save storage failed: {0}")]
    Storage(String),
}

pub trait SaveStorage {
    fn load_profile(&self) -> Result<Profile, SaveError>;
    fn save_profile(&self, profile: &Profile) -> Result<(), SaveError>;
}

/// Stores the profile as a JSON file on the local filesystem. This is one
/// possible `SaveStorage` implementation among others (SQLite, IndexedDB,
/// mobile-native storage, ...); callers decide which one to use.
pub struct FileSaveStorage {
    path: PathBuf,
}

impl FileSaveStorage {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl SaveStorage for FileSaveStorage {
    fn load_profile(&self) -> Result<Profile, SaveError> {
        if !self.path.exists() {
            return Err(SaveError::NotFound);
        }

        let raw =
            fs::read_to_string(&self.path).map_err(|err| SaveError::Storage(err.to_string()))?;
        serde_json::from_str(&raw).map_err(|err| SaveError::Storage(err.to_string()))
    }

    fn save_profile(&self, profile: &Profile) -> Result<(), SaveError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|err| SaveError::Storage(err.to_string()))?;
        }

        let raw = serde_json::to_string_pretty(profile)
            .map_err(|err| SaveError::Storage(err.to_string()))?;
        fs::write(&self.path, raw).map_err(|err| SaveError::Storage(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "deduced-save-test-{name}-{}.json",
            std::process::id()
        ))
    }

    #[test]
    fn load_profile_reports_not_found_before_any_save() {
        let storage = FileSaveStorage::new(temp_path("missing"));
        assert!(matches!(storage.load_profile(), Err(SaveError::NotFound)));
    }

    #[test]
    fn save_then_load_round_trips_the_profile() {
        let path = temp_path("roundtrip");
        let storage = FileSaveStorage::new(&path);

        let mut profile = Profile::default();
        profile.stats.record_round("cars", true, 120);

        storage.save_profile(&profile).expect("save should succeed");
        let loaded = storage.load_profile().expect("load should succeed");

        assert_eq!(loaded, profile);

        let _ = fs::remove_file(&path);
    }
}
