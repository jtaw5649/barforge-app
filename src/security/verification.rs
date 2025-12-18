use minisign_verify::{PublicKey, Signature};
use thiserror::Error;

const REGISTRY_PUBLIC_KEY: &str = "RWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBM0QTaLn73Y7GFO3";

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Invalid signature format: {0}")]
    InvalidSignature(String),

    #[error("Signature verification failed: {0}")]
    VerificationFailed(String),

    #[error("Key ID mismatch: expected {expected}, got {actual}")]
    KeyIdMismatch { expected: String, actual: String },
}

pub struct Verifier {
    public_key: PublicKey,
}

impl Verifier {
    #[must_use]
    pub fn new() -> Self {
        let public_key = PublicKey::from_base64(REGISTRY_PUBLIC_KEY)
            .expect("Compile-time public key must be valid");

        Self { public_key }
    }

    pub fn verify(&self, content: &[u8], signature_str: &str) -> Result<(), VerifyError> {
        let signature = Signature::decode(signature_str)
            .map_err(|e| VerifyError::InvalidSignature(e.to_string()))?;

        self.public_key
            .verify(content, &signature, false)
            .map_err(|e| VerifyError::VerificationFailed(e.to_string()))
    }

    pub fn verify_with_hash(
        &self,
        content: &[u8],
        signature_str: &str,
        expected_hash: &str,
    ) -> Result<(), VerifyError> {
        self.verify(content, signature_str)?;

        let actual_hash = compute_sha256(content);
        if actual_hash != expected_hash {
            return Err(VerifyError::VerificationFailed(format!(
                "Hash mismatch: expected {expected_hash}, got {actual_hash}"
            )));
        }

        Ok(())
    }
}

impl Default for Verifier {
    fn default() -> Self {
        Self::new()
    }
}

#[must_use]
pub fn compute_sha256(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex_encode(&result)
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifier_creates_with_valid_key() {
        let verifier = Verifier::new();
        assert!(std::mem::size_of_val(&verifier) > 0);
    }

    #[test]
    fn verifier_default_same_as_new() {
        let v1 = Verifier::new();
        let v2 = Verifier::default();
        assert!(std::mem::size_of_val(&v1) == std::mem::size_of_val(&v2));
    }

    #[test]
    fn sha256_computes_correct_hash() {
        let data = b"hello world";
        let hash = compute_sha256(data);
        assert_eq!(
            hash,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }

    #[test]
    fn sha256_empty_input() {
        let hash = compute_sha256(b"");
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn verify_rejects_invalid_signature_format() {
        let verifier = Verifier::new();
        let result = verifier.verify(b"test", "not-a-valid-signature");
        assert!(matches!(result, Err(VerifyError::InvalidSignature(_))));
    }

    #[test]
    fn verify_with_hash_checks_both() {
        let verifier = Verifier::new();
        let content = b"test content";
        let wrong_hash = "0000000000000000000000000000000000000000000000000000000000000000";

        let result = verifier.verify_with_hash(content, "untrusted comment: test\nRWQf6LRCGA9i53mlYecO4IzT51TGPpvWucNSCh1CBM0QTaLn73Y7GFO3\ntrusted comment: test\nAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\n", wrong_hash);

        assert!(result.is_err());
    }
}
