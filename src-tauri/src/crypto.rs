// Cryptographic signing for AI responses
// Ensures authenticity, integrity, and non-repudiation

use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Keypair for signing AI responses
#[derive(Clone)]
pub struct SigningIdentity {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

/// Signed message with verification data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedMessage {
    pub content: String,
    pub signature: String,  // Base64 encoded
    pub public_key: String, // Base64 encoded
    pub timestamp: u64,
}

impl SigningIdentity {
    /// Generate new keypair
    pub fn generate() -> Self {
        let mut csprng = OsRng;
        let signing_key = SigningKey::from_bytes(&rand::Rng::gen::<[u8; 32]>(&mut csprng));
        let verifying_key = signing_key.verifying_key();

        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Load keypair from file
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let bytes = fs::read(&path).map_err(|e| format!("Failed to read keypair: {}", e))?;

        if bytes.len() != 32 {
            return Err("Invalid keypair file (expected 32 bytes)".to_string());
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes);

        let signing_key = SigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();

        Ok(Self {
            signing_key,
            verifying_key,
        })
    }

    /// Save keypair to file
    pub fn save(&self, path: PathBuf) -> Result<(), String> {
        let bytes = self.signing_key.to_bytes();
        fs::write(&path, &bytes).map_err(|e| format!("Failed to write keypair: {}", e))?;

        Ok(())
    }

    /// Sign a message
    pub fn sign(&self, message: &str) -> SignedMessage {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create canonical representation for signing
        let canonical = format!("{}|{}", message, timestamp);

        let signature = self.signing_key.sign(canonical.as_bytes());

        SignedMessage {
            content: message.to_string(),
            signature: general_purpose::STANDARD.encode(signature.to_bytes()),
            public_key: general_purpose::STANDARD.encode(self.verifying_key.to_bytes()),
            timestamp,
        }
    }

    /// Get public key as base64
    pub fn public_key_base64(&self) -> String {
        general_purpose::STANDARD.encode(self.verifying_key.to_bytes())
    }

    /// Get public key bytes
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.verifying_key.to_bytes()
    }
}

/// Verify a signed message
pub fn verify_signed_message(signed_msg: &SignedMessage) -> Result<bool, String> {
    // Decode public key
    let public_key_bytes = general_purpose::STANDARD
        .decode(&signed_msg.public_key)
        .map_err(|e| format!("Invalid public key encoding: {}", e))?;

    if public_key_bytes.len() != 32 {
        return Err("Invalid public key length".to_string());
    }

    let mut key_bytes = [0u8; 32];
    key_bytes.copy_from_slice(&public_key_bytes);

    let verifying_key =
        VerifyingKey::from_bytes(&key_bytes).map_err(|e| format!("Invalid public key: {}", e))?;

    // Decode signature
    let signature_bytes = general_purpose::STANDARD
        .decode(&signed_msg.signature)
        .map_err(|e| format!("Invalid signature encoding: {}", e))?;

    if signature_bytes.len() != 64 {
        return Err("Invalid signature length".to_string());
    }

    let signature = Signature::from_bytes(&signature_bytes.try_into().unwrap());

    // Reconstruct canonical message
    let canonical = format!("{}|{}", signed_msg.content, signed_msg.timestamp);

    // Verify
    match verifying_key.verify(canonical.as_bytes(), &signature) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Generate fingerprint from public key (for display)
pub fn public_key_fingerprint(public_key_base64: &str) -> Result<String, String> {
    let bytes = general_purpose::STANDARD
        .decode(public_key_base64)
        .map_err(|e| format!("Invalid public key: {}", e))?;

    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(&bytes);
    let hex = format!("{:x}", hash);

    // Return first 16 chars as fingerprint
    Ok(hex[..16].to_uppercase())
}

/// Internal helper for HTTP API - verify signature with raw strings
#[allow(dead_code)]
pub fn verify_signature_internal(message: &str, signature: &str, public_key: &str) -> bool {
    // Decode signature and public key
    let sig_bytes = match general_purpose::STANDARD.decode(signature) {
        Ok(b) => b,
        Err(_) => return false,
    };

    let pk_bytes = match general_purpose::STANDARD.decode(public_key) {
        Ok(b) => b,
        Err(_) => return false,
    };

    if pk_bytes.len() != 32 || sig_bytes.len() != 64 {
        return false;
    }

    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&pk_bytes);

    let verifying_key = match VerifyingKey::from_bytes(&key_array) {
        Ok(k) => k,
        Err(_) => return false,
    };

    let signature = Signature::from_bytes(&sig_bytes.try_into().unwrap());

    verifying_key.verify(message.as_bytes(), &signature).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_identity() {
        let identity = SigningIdentity::generate();
        let public_key = identity.public_key_base64();

        assert!(!public_key.is_empty());
        assert_eq!(
            general_purpose::STANDARD.decode(&public_key).unwrap().len(),
            32
        );
    }

    #[test]
    fn test_sign_and_verify() {
        let identity = SigningIdentity::generate();
        let message = "This is a test AI response";

        let signed = identity.sign(message);

        assert_eq!(signed.content, message);
        assert!(!signed.signature.is_empty());
        assert!(!signed.public_key.is_empty());

        let verified = verify_signed_message(&signed).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_tamper_detection() {
        let identity = SigningIdentity::generate();
        let message = "Original message";

        let mut signed = identity.sign(message);

        // Tamper with content
        signed.content = "Tampered message".to_string();

        let verified = verify_signed_message(&signed).unwrap();
        assert!(!verified); // Should fail verification
    }

    #[test]
    fn test_wrong_signature() {
        let identity1 = SigningIdentity::generate();
        let identity2 = SigningIdentity::generate();

        let signed = identity1.sign("Test message");

        // Try to verify with wrong public key
        let mut fake_signed = signed.clone();
        fake_signed.public_key = identity2.public_key_base64();

        let verified = verify_signed_message(&fake_signed).unwrap();
        assert!(!verified);
    }

    #[test]
    fn test_save_and_load() {
        let identity = SigningIdentity::generate();
        let original_pubkey = identity.public_key_base64();

        let temp_path = PathBuf::from("/tmp/test_keypair.key");

        // Save
        identity.save(temp_path.clone()).unwrap();

        // Load
        let loaded = SigningIdentity::load(temp_path.clone()).unwrap();
        let loaded_pubkey = loaded.public_key_base64();

        assert_eq!(original_pubkey, loaded_pubkey);

        // Cleanup
        let _ = fs::remove_file(temp_path);
    }

    #[test]
    fn test_fingerprint() {
        let identity = SigningIdentity::generate();
        let pubkey = identity.public_key_base64();

        let fingerprint = public_key_fingerprint(&pubkey).unwrap();

        assert_eq!(fingerprint.len(), 16);
        assert!(fingerprint.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_deterministic_signature() {
        let identity = SigningIdentity::generate();
        let message = "Same message";

        let signed1 = identity.sign(message);

        // Wait to ensure different timestamp
        std::thread::sleep(std::time::Duration::from_secs(1));

        let signed2 = identity.sign(message);

        // Signatures should differ due to timestamp
        assert_ne!(signed1.signature, signed2.signature);
        assert_ne!(signed1.timestamp, signed2.timestamp);

        // But both should verify
        assert!(verify_signed_message(&signed1).unwrap());
        assert!(verify_signed_message(&signed2).unwrap());
    }
}
