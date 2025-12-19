use crate::security::{
    ExtractionError, OfflinePolicy, RevocationError, SCRIPT_TIMEOUT_SECS, ScriptError, Verifier,
    VerifyError, check_revocation, compute_sha256, extract_tarball_safe, run_script_sandboxed,
};
use crate::services::{DepReport, PackageConfigError, PackageToml, check_dependencies};
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InstallError {
    #[error("Revocation check failed: {0}")]
    RevocationCheck(#[from] RevocationError),

    #[error("Signature verification failed: {0}")]
    SignatureVerification(#[from] VerifyError),

    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Archive extraction failed: {0}")]
    Extraction(#[from] ExtractionError),

    #[error("Package config error: {0}")]
    PackageConfig(#[from] PackageConfigError),

    #[error("Missing required dependencies: {0:?}")]
    MissingDependencies(Vec<String>),

    #[error("Script execution failed: {0}")]
    ScriptExecution(#[from] ScriptError),

    #[error("Network error: {0}")]
    Network(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallStage {
    RevocationCheck,
    FetchingSignature,
    DownloadingPackage,
    VerifyingSignature,
    VerifyingHash,
    ExtractingPackage,
    CheckingDependencies,
    RunningInstallScript,
    Complete,
}

impl InstallStage {
    #[must_use]
    pub fn description(&self) -> &'static str {
        match self {
            Self::RevocationCheck => "Checking revocation status",
            Self::FetchingSignature => "Fetching signature",
            Self::DownloadingPackage => "Downloading package",
            Self::VerifyingSignature => "Verifying signature",
            Self::VerifyingHash => "Verifying hash",
            Self::ExtractingPackage => "Extracting package",
            Self::CheckingDependencies => "Checking dependencies",
            Self::RunningInstallScript => "Running install script",
            Self::Complete => "Installation complete",
        }
    }
}

pub struct InstallParams<'a> {
    pub uuid: &'a str,
    pub version: &'a str,
    pub package_data: &'a [u8],
    pub signature: &'a str,
    pub expected_hash: &'a str,
    pub dest_dir: &'a Path,
}

pub struct SecureInstaller {
    verifier: Verifier,
    offline_policy: OfflinePolicy,
}

impl SecureInstaller {
    #[must_use]
    pub fn new() -> Self {
        Self {
            verifier: Verifier::new(),
            offline_policy: OfflinePolicy::default(),
        }
    }

    #[must_use]
    pub fn with_offline_policy(mut self, policy: OfflinePolicy) -> Self {
        self.offline_policy = policy;
        self
    }

    pub async fn install<F>(
        &self,
        params: InstallParams<'_>,
        progress: F,
    ) -> Result<InstallResult, InstallError>
    where
        F: Fn(InstallStage),
    {
        progress(InstallStage::RevocationCheck);
        check_revocation(params.uuid, params.version, self.offline_policy).await?;

        progress(InstallStage::VerifyingSignature);
        self.verifier
            .verify(params.package_data, params.signature)?;

        progress(InstallStage::VerifyingHash);
        let actual_hash = compute_sha256(params.package_data);
        if actual_hash != params.expected_hash {
            return Err(InstallError::HashMismatch {
                expected: params.expected_hash.to_string(),
                actual: actual_hash,
            });
        }

        progress(InstallStage::ExtractingPackage);
        std::fs::create_dir_all(params.dest_dir)?;
        extract_tarball_safe(params.package_data, params.dest_dir)?;

        let package_toml_path = params.dest_dir.join("Package.toml");
        let package_config = if package_toml_path.exists() {
            Some(PackageToml::from_file(&package_toml_path)?)
        } else {
            None
        };

        progress(InstallStage::CheckingDependencies);
        let dep_report = if let Some(ref config) = package_config {
            let specs = config.to_dep_specs();
            let report = check_dependencies(&specs);
            if !report.all_satisfied {
                return Err(InstallError::MissingDependencies(
                    report.missing_required.clone(),
                ));
            }
            Some(report)
        } else {
            None
        };

        progress(InstallStage::RunningInstallScript);
        let script_result = if let Some(ref config) = package_config {
            if let Some(ref script_name) = config.package.install_script {
                let script_path = params.dest_dir.join(script_name);
                if script_path.exists() {
                    let sandbox_config = config.to_sandbox_config();
                    let timeout = Duration::from_secs(SCRIPT_TIMEOUT_SECS);
                    Some(run_script_sandboxed(
                        &script_path,
                        params.dest_dir,
                        &sandbox_config,
                        timeout,
                    )?)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        progress(InstallStage::Complete);

        Ok(InstallResult {
            module_dir: params.dest_dir.to_path_buf(),
            dep_report,
            script_output: script_result.map(|r| r.stdout),
        })
    }

    pub fn verify_only(
        &self,
        package_data: &[u8],
        signature: &str,
        expected_hash: &str,
    ) -> Result<(), InstallError> {
        self.verifier.verify(package_data, signature)?;

        let actual_hash = compute_sha256(package_data);
        if actual_hash != expected_hash {
            return Err(InstallError::HashMismatch {
                expected: expected_hash.to_string(),
                actual: actual_hash,
            });
        }

        Ok(())
    }
}

impl Default for SecureInstaller {
    fn default() -> Self {
        Self::new()
    }
}

pub struct InstallResult {
    pub module_dir: PathBuf,
    pub dep_report: Option<DepReport>,
    pub script_output: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_stage_descriptions() {
        assert!(!InstallStage::RevocationCheck.description().is_empty());
        assert!(!InstallStage::FetchingSignature.description().is_empty());
        assert!(!InstallStage::DownloadingPackage.description().is_empty());
        assert!(!InstallStage::VerifyingSignature.description().is_empty());
        assert!(!InstallStage::VerifyingHash.description().is_empty());
        assert!(!InstallStage::ExtractingPackage.description().is_empty());
        assert!(!InstallStage::CheckingDependencies.description().is_empty());
        assert!(!InstallStage::RunningInstallScript.description().is_empty());
        assert!(!InstallStage::Complete.description().is_empty());
    }

    #[test]
    fn secure_installer_default() {
        let installer = SecureInstaller::default();
        assert_eq!(installer.offline_policy, OfflinePolicy::FailClosed);
    }

    #[test]
    fn secure_installer_with_offline_policy() {
        let installer = SecureInstaller::new().with_offline_policy(OfflinePolicy::AllowOffline);
        assert_eq!(installer.offline_policy, OfflinePolicy::AllowOffline);
    }

    #[test]
    fn verify_only_rejects_hash_mismatch() {
        let installer = SecureInstaller::new();
        let data = b"test data";
        let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";

        let result = installer.verify_only(data, "invalid-sig", wrong_hash);
        assert!(result.is_err());
    }
}
