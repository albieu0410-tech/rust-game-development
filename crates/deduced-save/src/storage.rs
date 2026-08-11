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
