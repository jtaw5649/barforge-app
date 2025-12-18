pub mod archive_extraction;
mod path_validation;
pub mod revocation;
pub mod sandbox;
pub mod script_execution;
mod script_inspection;
mod url_validation;
pub mod verification;

pub use path_validation::{validate_extraction_path, PathTraversalError};
pub use sandbox::{
    apply as apply_sandbox, is_allowed_read_path, is_allowed_write_path, is_available as is_sandbox_available,
    SandboxConfig, SandboxResult, SandboxSeverity, SandboxStatus,
};
pub use script_inspection::{inspect_script_safety, RiskyPattern, ScriptInspectionResult};
pub use url_validation::{
    parse_github_url_safe, validate_github_url, validate_web_url, UrlValidationError,
};
pub use verification::{compute_sha256, VerifyError, Verifier};
pub use revocation::{check_revocation, OfflinePolicy, RevocationError};
pub use archive_extraction::{
    extract_tarball_safe, extract_tarball_from_reader, normalize_path_algebraic,
    safe_extraction_path, ExtractionError, MAX_PACKAGE_SIZE,
};
pub use script_execution::{
    run_script_sandboxed, run_script_unsandboxed, ScriptError, ScriptResult,
    SCRIPT_TIMEOUT_SECS,
};
