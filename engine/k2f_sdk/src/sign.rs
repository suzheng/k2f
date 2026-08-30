use crate::error::{map_compile_message, AgentError, INVALID_ARGUMENT};
use k2f_package::{generate_secret_key, pack_bytes, sign_package, unpack_bytes, SecretKey};

pub struct GeneratedKey {
    pub secret_hex: String,
    pub public_hex: String,
    pub fingerprint: String,
}

pub fn generate_key() -> Result<GeneratedKey, AgentError> {
    let key = generate_secret_key().map_err(AgentError::from)?;
    Ok(GeneratedKey {
        secret_hex: key.to_hex(),
        public_hex: key.public_hex(),
        fingerprint: key.fingerprint(),
    })
}

/// Sign a compiled package. Agents save unsigned; a human/org key signs the lock bytes.
/// `signed_at` is UTC unix seconds. None uses the current UTC time.
pub fn sign(
    package_bytes: &[u8],
    secret_hex: &str,
    signed_by: Option<&str>,
    signed_at: Option<i64>,
) -> Result<Vec<u8>, AgentError> {
    let mut pkg = unpack_bytes(package_bytes).map_err(AgentError::from)?;
    let key = SecretKey::from_hex(secret_hex)
        .map_err(|e| AgentError::new(INVALID_ARGUMENT, e.to_string()))?;
    let signed_at = signed_at.unwrap_or_else(k2f_package::utc_unix_seconds);
    sign_package(&mut pkg, &key, signed_by, signed_at).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("UNLOCKED") || msg.contains("refuse to sign") {
            AgentError::new(INVALID_ARGUMENT, msg)
        } else {
            map_compile_message(&msg)
        }
    })?;
    pack_bytes(&pkg).map_err(AgentError::from)
}
