use crate::error::{
    PackageError, VerifyStatus, CODE_SIGNED, CODE_SIGNED_BUT_BROKEN, CODE_UNKNOWN_PAINT_OP,
    CODE_UNSIGNED,
};
use crate::package::Package;
use crate::sign::key::fingerprint_from_public_hex;
use crate::sign::{parse_valid_signature_file, signature_matches_lock, SignatureFile};
use crate::verify::verify_package;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityStatus {
    Unlocked,
    Unsigned,
    Signed,
    SignedButBroken,
    ContentChanged,
    AppearanceChanged,
    EngineMismatch,
    FontMissing,
    UnknownPaintOp,
}

impl IntegrityStatus {
    pub fn code(self) -> &'static str {
        match self {
            Self::Unlocked => crate::error::CODE_UNLOCKED,
            Self::Unsigned => CODE_UNSIGNED,
            Self::Signed => CODE_SIGNED,
            Self::SignedButBroken => CODE_SIGNED_BUT_BROKEN,
            Self::ContentChanged => crate::error::CODE_CONTENT_CHANGED,
            Self::AppearanceChanged => crate::error::CODE_APPEARANCE_CHANGED,
            Self::EngineMismatch => crate::error::CODE_ENGINE_MISMATCH,
            Self::FontMissing => crate::error::CODE_FONT_MISSING,
            Self::UnknownPaintOp => CODE_UNKNOWN_PAINT_OP,
        }
    }

    pub fn is_ok(self) -> bool {
        matches!(self, Self::Unsigned | Self::Signed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntegrityReport {
    pub status: IntegrityStatus,
    pub hash: VerifyStatus,
    pub fingerprint: Option<String>,
    pub signed_by: Option<String>,
    pub signed_at: Option<i64>,
    pub generated_by: Option<String>,
}

pub fn inspect_package(package: &Package) -> Result<IntegrityReport, PackageError> {
    let hash = verify_package(package)?;
    let unknown_ops = package.lock_json.as_ref().and_then(|j| {
        serde_json::from_str::<k2f_core::LockFile>(j)
            .ok()
            .map(|l| l.has_unknown_paint_ops())
    }) == Some(true);

    let parsed = package
        .signatures_json
        .as_deref()
        .map(parse_valid_signature_file)
        .transpose();
    let file = match parsed {
        Ok(f) => f,
        Err(_) => {
            return Ok(report(
                IntegrityStatus::SignedButBroken,
                hash,
                None,
                package,
            ));
        }
    };

    let crypto_ok = match (&file, package.lock_json.as_deref()) {
        (Some(file), Some(lock)) => signature_matches_lock(file, lock),
        (Some(_), None) => false,
        (None, _) => false,
    };
    let meta = file.as_ref().and_then(SignatureFile::first);

    let has_sig = file.is_some();
    let status = if has_sig {
        if hash == VerifyStatus::Valid && crypto_ok && !unknown_ops {
            IntegrityStatus::Signed
        } else {
            IntegrityStatus::SignedButBroken
        }
    } else if unknown_ops && hash == VerifyStatus::Valid {
        IntegrityStatus::UnknownPaintOp
    } else {
        match hash {
            VerifyStatus::Valid => IntegrityStatus::Unsigned,
            VerifyStatus::Unlocked => IntegrityStatus::Unlocked,
            VerifyStatus::ContentChanged => IntegrityStatus::ContentChanged,
            VerifyStatus::AppearanceChanged => IntegrityStatus::AppearanceChanged,
            VerifyStatus::EngineMismatch => IntegrityStatus::EngineMismatch,
            VerifyStatus::FontMissing => IntegrityStatus::FontMissing,
        }
    };

    Ok(report(status, hash, meta, package))
}

fn report(
    status: IntegrityStatus,
    hash: VerifyStatus,
    meta: Option<&crate::sign::SignatureRecord>,
    package: &Package,
) -> IntegrityReport {
    IntegrityReport {
        status,
        hash,
        fingerprint: meta.map(|m| {
            fingerprint_from_public_hex(&m.public_key).unwrap_or_else(|_| m.public_key.clone())
        }),
        signed_by: meta.and_then(|m| m.signed_by.clone()),
        signed_at: meta.map(|m| m.signed_at),
        generated_by: package.manifest.generated_by.clone(),
    }
}
