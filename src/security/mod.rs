pub mod archive_extraction;
mod path_validation;
pub mod revocation;
pub mod sandbox;
pub mod script_execution;
mod script_inspection;
mod url_validation;
pub mod verification;

pub use archive_extraction::{
    ExtractionError, MAX_PACKAGE_SIZE, extract_tarball_from_reader, extract_tarball_safe,
    normalize_path_algebraic, safe_extraction_path,
};
pub use path_validation::{PathTraversalError, validate_extraction_path};
pub use revocation::{OfflinePolicy, RevocationError, check_revocation};
pub use sandbox::{
    SandboxConfig, SandboxResult, SandboxSeverity, SandboxStatus, apply as apply_sandbox,
    is_allowed_read_path, is_allowed_write_path, is_available as is_sandbox_available,
};
pub use script_execution::{
    SCRIPT_TIMEOUT_SECS, ScriptError, ScriptResult, run_script_sandboxed, run_script_unsandboxed,
};
pub use script_inspection::{RiskyPattern, ScriptInspectionResult, inspect_script_safety};
pub use url_validation::{
    UrlValidationError, parse_github_url_safe, validate_github_url, validate_web_url,
};
pub use verification::{Verifier, VerifyError, compute_sha256};
