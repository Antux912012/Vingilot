use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use sha2::{Digest, Sha256};

use crate::error::{CoreError, CoreResult};

const ENCRYPTED_MAGIC_HEADER: &[u8] = b"VINGILOT_ENCRYPTED_V1\n";

/// Derives a 256-bit symmetric key from user passphrase and salt using SHA-256 rounds.
fn derive_key(passphrase: &str, salt: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(passphrase.as_bytes());
    hasher.update(salt);
    for _ in 0..10_000 {
        let digest = hasher.finalize_reset();
        hasher.update(digest);
        hasher.update(passphrase.as_bytes());
    }
    hasher.finalize().into()
}

/// Encrypts plaintext bytes using passphrase-derived AES-256-GCM.
pub fn encrypt_bundle_data(plaintext: &[u8], passphrase: &str) -> CoreResult<Vec<u8>> {
    let salt: [u8; 16] = [
        17, 42, 99, 12, 55, 66, 77, 88, 91, 102, 113, 124, 135, 146, 157, 168,
    ];
    let key_bytes = derive_key(passphrase, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| CoreError::Export(format!("Cipher init error: {e}")))?;

    let nonce_bytes: [u8; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CoreError::Export(format!("Encryption error: {e}")))?;

    let mut output = Vec::with_capacity(ENCRYPTED_MAGIC_HEADER.len() + ciphertext.len());
    output.extend_from_slice(ENCRYPTED_MAGIC_HEADER);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

/// Determines whether data has the encrypted bundle magic header.
pub fn is_bundle_encrypted(data: &[u8]) -> bool {
    data.starts_with(ENCRYPTED_MAGIC_HEADER)
}

/// Decrypts ciphertext bytes using passphrase-derived AES-256-GCM.
pub fn decrypt_bundle_data(data: &[u8], passphrase: &str) -> CoreResult<Vec<u8>> {
    if !is_bundle_encrypted(data) {
        return Ok(data.to_vec());
    }

    let raw_ciphertext = &data[ENCRYPTED_MAGIC_HEADER.len()..];
    let salt: [u8; 16] = [
        17, 42, 99, 12, 55, 66, 77, 88, 91, 102, 113, 124, 135, 146, 157, 168,
    ];
    let key_bytes = derive_key(passphrase, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| CoreError::Import(format!("Cipher init error: {e}")))?;

    let nonce_bytes: [u8; 12] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
    let nonce = Nonce::from_slice(&nonce_bytes);

    cipher.decrypt(nonce, raw_ciphertext).map_err(|_| {
        CoreError::Import("Invalid passphrase or corrupted encrypted bundle".to_string())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_and_decryption_lifecycle() {
        let secret = b"Sample Vingilot Sync Archive Payload";
        let passphrase = "correct-horse-battery-staple";

        let encrypted = encrypt_bundle_data(secret, passphrase).unwrap();
        assert!(is_bundle_encrypted(&encrypted));

        // Decrypt with correct passphrase
        let decrypted = decrypt_bundle_data(&encrypted, passphrase).unwrap();
        assert_eq!(decrypted, secret);

        // Decrypt with incorrect passphrase fails
        let failed = decrypt_bundle_data(&encrypted, "wrong-password");
        assert!(failed.is_err());
    }
}
