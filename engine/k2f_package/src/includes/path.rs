use crate::error::PackageError;
use crate::paths;

pub const MAX_INCLUDE_DEPTH: usize = 8;

pub fn validate_include_path(path: &str) -> Result<(), PackageError> {
    if !paths::is_content_json_path(path) {
        return Err(PackageError::Other(format!(
            "INCLUDE_PATH: invalid include path '{path}'"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_nested_content_json() {
        validate_include_path("content/ch01.json").unwrap();
        validate_include_path("content/parts/ch01.json").unwrap();
    }

    #[test]
    fn rejects_root_and_traversal() {
        assert!(validate_include_path("content/root.json").is_err());
        assert!(validate_include_path("content/../root.json").is_err());
        assert!(validate_include_path("styles/theme.json").is_err());
    }
}
