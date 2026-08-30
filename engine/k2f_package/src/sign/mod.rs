pub(crate) mod hex;
pub(crate) mod key;
mod record;
mod sign;

pub use key::{generate_secret_key, utc_unix_seconds, SecretKey};
pub use record::{SignatureFile, SignatureRecord};
pub use sign::{parse_valid_signature_file, sign_package, signature_matches_lock};
