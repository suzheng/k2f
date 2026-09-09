use crate::error::PackageError;
use crate::package::Package;
use crate::schema::validate_signatures_json;
use crate::sign::hex;
use crate::sign::key::{verify_signature, SecretKey};
use crate::sign::record::{SignatureFile, SignatureRecord, ALG_ED25519};
use crate::verify::verify_package;
use k2f_core::canonical_json_string;
use serde_json::Value;

pub fn sign_package(
    package: &mut Package,
    secret: &SecretKey,
    signed_by: Option<&str>,
    signed_at: i64,
) -> Result<(), PackageError> {
    if signed_at < 0 {
        return Err(PackageError::Other(
            "signed_at must be UTC unix seconds >= 0".into(),
        ));
    }
    let hash = verify_package(package)?;
    if !hash.is_self_consistent() {
        return Err(PackageError::Other(format!(
            "refuse to sign: {} (lock must match content and appearance)",
            hash.code()
        )));
    }
    if package.lock_json.as_ref().and_then(|l| {
        serde_json::from_str::<k2f_core::LockFile>(l)
            .ok()
            .map(|lock| lock.has_unknown_paint_ops())
    }) == Some(true)
    {
        return Err(PackageError::Other(
            "refuse to sign: UNKNOWN_PAINT_OP".into(),
        ));
    }
    let lock_json = package
        .lock_json
        .as_ref()
        .ok_or_else(|| PackageError::Other("UNLOCKED: cannot sign a draft".into()))?;
    let sig = secret.sign(lock_json.as_bytes());
    let file = SignatureFile::single(SignatureRecord {
        alg: ALG_ED25519.to_string(),
        public_key: secret.public_hex(),
        signature: hex::encode(&sig),
        signed_at,
        signed_by: signed_by.map(|s| s.to_string()),
    });
    package.signatures_json =
        Some(canonical_json_string(&file).map_err(|e| PackageError::Other(e.to_string()))?);
    Ok(())
}

fn parse_signature_file(json: &str) -> Result<SignatureFile, PackageError> {
    serde_json::from_str(json).map_err(|e| PackageError::Other(format!("signatures: {e}")))
}

/// Schema + shape. Inspect uses this so a v2 or extra-field file cannot green-check.
pub fn parse_valid_signature_file(json: &str) -> Result<SignatureFile, PackageError> {
    let v: Value =
        serde_json::from_str(json).map_err(|e| PackageError::Other(format!("signatures: {e}")))?;
    validate_signatures_json(&v)?;
    parse_signature_file(json)
}

pub fn signature_matches_lock(file: &SignatureFile, lock_json: &str) -> bool {
    if file.version != 1 || file.signatures.len() != 1 {
        return false;
    }
    let Some(record) = file.first() else {
        return false;
    };
    if record.alg != ALG_ED25519 {
        return false;
    }
    verify_signature(&record.public_key, &record.signature, lock_json.as_bytes()).is_ok()
}
