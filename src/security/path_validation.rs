use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum PathTraversalError {
    #[error("path contains parent directory reference (..)")]
    ParentDirectoryReference,
    #[error("path escapes base directory")]
    EscapesBaseDirectory,
    #[error("path is absolute")]
    AbsolutePath,
    #[error("IO error: {0}")]
    IoError(String),
}

pub fn validate_extraction_path(
    base_dir: &Path,
    relative_path: &Path,
) -> Result<PathBuf, PathTraversalError> {
    if relative_path.is_absolute() {
        return Err(PathTraversalError::AbsolutePath);
    }

    for component in relative_path.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(PathTraversalError::ParentDirectoryReference);
        }
    }

    let dest_path = base_dir.join(relative_path);

    let canonical_base = base_dir
        .canonicalize()
        .map_err(|e| PathTraversalError::IoError(e.to_string()))?;

    let mut check_path = dest_path.clone();
    while !check_path.exists() {
        match check_path.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => {
                check_path = parent.to_path_buf();
            }
            _ => break,
        }
    }

    if check_path.exists() {
        let canonical_check = check_path
            .canonicalize()
            .map_err(|e| PathTraversalError::IoError(e.to_string()))?;

        if !canonical_check.starts_with(&canonical_base) {
            return Err(PathTraversalError::EscapesBaseDirectory);
        }
    }

    Ok(dest_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        TempDir::new().expect("Failed to create temp dir")
    }

    #[test]
    fn rejects_path_with_parent_reference() {
        let base = setup_test_dir();
        let result = validate_extraction_path(base.path(), Path::new("../escape"));
        assert_eq!(result, Err(PathTraversalError::ParentDirectoryReference));
    }

    #[test]
    fn rejects_path_with_nested_parent_reference() {
        let base = setup_test_dir();
        let result = validate_extraction_path(base.path(), Path::new("foo/../../../escape"));
        assert_eq!(result, Err(PathTraversalError::ParentDirectoryReference));
    }

    #[test]
    fn rejects_absolute_path() {
        let base = setup_test_dir();
        let result = validate_extraction_path(base.path(), Path::new("/etc/passwd"));
        assert_eq!(result, Err(PathTraversalError::AbsolutePath));
    }

    #[test]
    fn accepts_valid_relative_path() {
        let base = setup_test_dir();
        let result = validate_extraction_path(base.path(), Path::new("config.json"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), base.path().join("config.json"));
    }

    #[test]
    fn accepts_valid_nested_path() {
        let base = setup_test_dir();
        let result = validate_extraction_path(base.path(), Path::new("subdir/file.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), base.path().join("subdir/file.txt"));
    }

    #[test]
    fn accepts_path_with_current_dir_reference() {
        let base = setup_test_dir();
        let result = validate_extraction_path(base.path(), Path::new("./config.json"));
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_path_that_escapes_after_canonicalization() {
        let base = setup_test_dir();
        fs::create_dir_all(base.path().join("subdir")).unwrap();
        let result = validate_extraction_path(base.path(), Path::new("subdir/../../escape"));
        assert_eq!(result, Err(PathTraversalError::ParentDirectoryReference));
    }
}
