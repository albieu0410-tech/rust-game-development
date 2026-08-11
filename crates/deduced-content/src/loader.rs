use std::{fs, path::Path};

use deduced_core::{Answer, CategoryDefinition, GameContent};
use thiserror::Error;

use crate::validation::{ContentValidationError, validate_content};

#[derive(Debug, Error)]
pub enum ContentLoadError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse {path}: {source}")]
    Parse {
        path: String,
        source: serde_json::Error,
    },
    #[error(transparent)]
    Validation(#[from] ContentValidationError),
}

pub fn load_content_from_dir(root: impl AsRef<Path>) -> Result<GameContent, ContentLoadError> {
    let root = root.as_ref();
    let categories_dir = root.join("categories");
    let answers_dir = root.join("answers");

    let categories = load_json_files::<CategoryDefinition>(&categories_dir)?;
    let answers = load_json_files::<Vec<Answer>>(&answers_dir)?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let content = GameContent {
        categories,
        answers,
        content_version: "dev".to_string(),
    };

    validate_content(&content)?;
    Ok(content)
}

fn load_json_files<T>(dir: &Path) -> Result<Vec<T>, ContentLoadError>
where
    T: serde::de::DeserializeOwned,
{
    let entries = fs::read_dir(dir).map_err(|source| ContentLoadError::Read {
        path: dir.display().to_string(),
        source,
    })?;

    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| ContentLoadError::Read {
            path: dir.display().to_string(),
            source,
        })?;
        paths.push(entry.path());
    }
    paths.sort();

    let mut values = Vec::new();
    for path in paths {
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }

        let raw = fs::read_to_string(&path).map_err(|source| ContentLoadError::Read {
            path: path.display().to_string(),
            source,
        })?;
        values.push(
            serde_json::from_str(&raw).map_err(|source| ContentLoadError::Parse {
                path: path.display().to_string(),
                source,
            })?,
        );
    }

    Ok(values)
}
