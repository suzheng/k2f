use crate::error::PackageError;
use crate::sign::hex;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use k2f_core::sha256_hex;

#[derive(Clone)]
pub struct SecretKey {
    inner: SigningKey,
}

impl SecretKey {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self {
            inner: SigningKey::from_bytes(&bytes),
        }
    }

    pub fn from_hex(s: &str) -> Result<Self, PackageError> {
        let bytes = hex::decode(s).map_err(|e| PackageError::Other(format!("secret key: {e}")))?;
        let bytes: [u8; 32] = bytes.try_into().map_err(|_| {
            PackageError::Other("secret key must be 32 bytes (64 hex chars)".into())
        })?;
        Ok(Self::from_bytes(bytes))
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.inner.to_bytes())
    }

    pub fn public_hex(&self) -> String {
        hex::encode(self.inner.verifying_key().as_bytes())
    }

    pub fn fingerprint(&self) -> String {
        fingerprint_public(&self.inner.verifying_key().to_bytes())
    }

    pub(crate) fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.inner.sign(message).to_bytes()
    }
}

pub fn generate_secret_key() -> Result<SecretKey, PackageError> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| PackageError::Other(format!("rng: {e}")))?;
    Ok(SecretKey::from_bytes(bytes))
}

pub fn utc_unix_seconds() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

pub fn fingerprint_public(public: &[u8; 32]) -> String {
    sha256_hex(public)
}

pub fn fingerprint_from_public_hex(s: &str) -> Result<String, PackageError> {
    let bytes = hex::decode(s).map_err(PackageError::Other)?;
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| PackageError::Other("public key must be 32 bytes".into()))?;
    Ok(fingerprint_public(&bytes))
}

pub fn verify_signature(
    public_hex: &str,
    signature_hex: &str,
    message: &[u8],
) -> Result<(), PackageError> {
    let public = hex::decode(public_hex).map_err(|e| PackageError::Other(e))?;
    let public: [u8; 32] = public
        .try_into()
        .map_err(|_| PackageError::Other("public key must be 32 bytes".into()))?;
    let sig = hex::decode(signature_hex).map_err(|e| PackageError::Other(e))?;
    let sig: [u8; 64] = sig
        .try_into()
        .map_err(|_| PackageError::Other("signature must be 64 bytes".into()))?;
    let vk = VerifyingKey::from_bytes(&public)
        .map_err(|e| PackageError::Other(format!("public key: {e}")))?;
    vk.verify(message, &Signature::from_bytes(&sig))
        .map_err(|_| PackageError::Other("signature does not match lock bytes".into()))
}
