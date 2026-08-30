use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=K2F_ENGINE_COMMIT_SHA");
    println!("cargo:rerun-if-changed=.git/HEAD");
    let profile = std::env::var("PROFILE").unwrap_or_default();
    let sha = std::env::var("K2F_ENGINE_COMMIT_SHA")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            if profile == "release" {
                git_sha().unwrap_or_else(|| {
                    panic!(
                        "release build requires K2F_ENGINE_COMMIT_SHA or a git checkout; got UNKNOWN"
                    )
                })
            } else {
                "UNKNOWN".to_string()
            }
        });
    if profile == "release" && (sha.is_empty() || sha == "UNKNOWN") {
        panic!("release build must not use engine_commit_sha=UNKNOWN");
    }
    println!("cargo:rustc-env=K2F_ENGINE_COMMIT_SHA={sha}");
}

fn git_sha() -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
