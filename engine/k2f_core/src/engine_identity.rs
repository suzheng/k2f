pub fn engine_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn engine_commit_sha() -> &'static str {
    env!("K2F_ENGINE_COMMIT_SHA")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_build_uses_unknown_unless_env_set() {
        let sha = engine_commit_sha();
        assert!(!sha.is_empty());
        if std::env::var("K2F_ENGINE_COMMIT_SHA")
            .ok()
            .filter(|s| !s.is_empty())
            .is_none()
        {
            assert_eq!(sha, "UNKNOWN");
        }
    }
}
