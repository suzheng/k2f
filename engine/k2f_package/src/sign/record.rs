use serde::{Deserialize, Serialize};

pub const ALG_ED25519: &str = "ed25519";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignatureFile {
    pub version: u32,
    pub signatures: Vec<SignatureRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SignatureRecord {
    pub alg: String,
    pub public_key: String,
    pub signature: String,
    pub signed_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_by: Option<String>,
}

impl SignatureFile {
    pub fn single(record: SignatureRecord) -> Self {
        Self {
            version: 1,
            signatures: vec![record],
        }
    }

    pub fn first(&self) -> Option<&SignatureRecord> {
        self.signatures.first()
    }
}
