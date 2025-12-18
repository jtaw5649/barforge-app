use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path, PathBuf};
use tar::Archive;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExtractionError {
    #[error("Path traversal detected in archive entry: {0}")]
    PathTraversal(String),

    #[error("Symlink not allowed in archive: {0}")]
    SymlinkNotAllowed(String),

    #[error("Hardlink not allowed in archive: {0}")]
    HardlinkNotAllowed(String),

    #[error("Absolute path in archive: {0}")]
    AbsolutePath(String),

    #[error("Invalid path component in archive: {0}")]
    InvalidPathComponent(String),

    #[error("Archive too large: {size} bytes exceeds {max} bytes")]
    TooLarge { size: u64, max: u64 },

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
}

pub const MAX_PACKAGE_SIZE: u64 = 50 * 1024 * 1024;

#[must_use]
pub fn normalize_path_algebraic(path: &Path) -> Option<PathBuf> {
    let mut components = Vec::new();

    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => return None,
            Component::CurDir => {}
            Component::ParentDir => {
                components.pop()?;
            }
            Component::Normal(c) => {
                if c.to_string_lossy().contains('\0') {
                    return None;
                }
                components.push(c);
            }
        }
    }

    if components.is_empty() {
        return None;
    }

    Some(components.iter().collect())
}

pub fn safe_extraction_path(base: &Path, relative: &Path) -> Result<PathBuf, ExtractionError> {
    let normalized = normalize_path_algebraic(relative)
        .ok_or_else(|| ExtractionError::PathTraversal(relative.display().to_string()))?;

    Ok(base.join(normalized))
}

pub fn extract_tarball_safe(data: &[u8], dest: &Path) -> Result<(), ExtractionError> {
    if data.len() as u64 > MAX_PACKAGE_SIZE {
        return Err(ExtractionError::TooLarge {
            size: data.len() as u64,
            max: MAX_PACKAGE_SIZE,
        });
    }

    let decoder = GzDecoder::new(data);
    let mut archive = Archive::new(decoder);

    fs::create_dir_all(dest)?;

    for entry in archive.entries()? {
        let mut entry = entry?;
        let entry_path = entry.path()?;
        let entry_path = entry_path.as_ref();

        if entry.header().entry_type().is_symlink() {
            return Err(ExtractionError::SymlinkNotAllowed(
                entry_path.display().to_string(),
            ));
        }

        if entry.header().entry_type().is_hard_link() {
            return Err(ExtractionError::HardlinkNotAllowed(
                entry_path.display().to_string(),
            ));
        }

        let safe_path = safe_extraction_path(dest, entry_path)?;

        if let Some(parent) = safe_path.parent() {
            fs::create_dir_all(parent)?;
        }

        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&safe_path)?;
        } else if entry.header().entry_type().is_file() {
            let mut file = File::create(&safe_path)?;
            io::copy(&mut entry, &mut file)?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(mode) = entry.header().mode() {
                    let permissions = fs::Permissions::from_mode(mode & 0o755);
                    fs::set_permissions(&safe_path, permissions)?;
                }
            }
        }
    }

    Ok(())
}

pub fn extract_tarball_from_reader<R: Read>(
    reader: R,
    dest: &Path,
    max_size: u64,
) -> Result<(), ExtractionError> {
    let mut data = Vec::new();
    let mut limited_reader = reader.take(max_size + 1);
    limited_reader.read_to_end(&mut data)?;

    if data.len() as u64 > max_size {
        return Err(ExtractionError::TooLarge {
            size: data.len() as u64,
            max: max_size,
        });
    }

    extract_tarball_safe(&data, dest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algebraic_normalization_handles_simple_path() {
        let path = Path::new("foo/bar/baz");
        let result = normalize_path_algebraic(path);
        assert_eq!(result, Some(PathBuf::from("foo/bar/baz")));
    }

    #[test]
    fn algebraic_normalization_handles_current_dir() {
        let path = Path::new("./foo/./bar");
        let result = normalize_path_algebraic(path);
        assert_eq!(result, Some(PathBuf::from("foo/bar")));
    }

    #[test]
    fn algebraic_normalization_resolves_parent_within_bounds() {
        let path = Path::new("foo/bar/../baz");
        let result = normalize_path_algebraic(path);
        assert_eq!(result, Some(PathBuf::from("foo/baz")));
    }

    #[test]
    fn algebraic_normalization_rejects_escape() {
        let path = Path::new("foo/../..");
        let result = normalize_path_algebraic(path);
        assert!(result.is_none());
    }

    #[test]
    fn algebraic_normalization_rejects_absolute() {
        let path = Path::new("/etc/passwd");
        let result = normalize_path_algebraic(path);
        assert!(result.is_none());
    }

    #[test]
    fn algebraic_normalization_rejects_initial_parent() {
        let path = Path::new("../escape");
        let result = normalize_path_algebraic(path);
        assert!(result.is_none());
    }

    #[test]
    fn algebraic_normalization_rejects_null_bytes() {
        let path = Path::new("foo\0bar");
        let result = normalize_path_algebraic(path);
        assert!(result.is_none());
    }

    #[test]
    fn algebraic_normalization_rejects_empty_result() {
        let path = Path::new("foo/..");
        let result = normalize_path_algebraic(path);
        assert!(result.is_none());
    }

    #[test]
    fn safe_extraction_path_joins_correctly() {
        let base = Path::new("/tmp/extract");
        let relative = Path::new("module/config.json");
        let result = safe_extraction_path(base, relative).unwrap();
        assert_eq!(result, PathBuf::from("/tmp/extract/module/config.json"));
    }

    #[test]
    fn safe_extraction_path_rejects_traversal() {
        let base = Path::new("/tmp/extract");
        let relative = Path::new("../../../etc/passwd");
        let result = safe_extraction_path(base, relative);
        assert!(matches!(result, Err(ExtractionError::PathTraversal(_))));
    }

    #[test]
    fn size_limit_enforced() {
        let data = vec![0u8; (MAX_PACKAGE_SIZE + 1) as usize];
        let result = extract_tarball_safe(&data, Path::new("/tmp/test"));
        assert!(matches!(result, Err(ExtractionError::TooLarge { .. })));
    }
}
