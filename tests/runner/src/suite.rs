use anyhow::{Context, Result};
use colored::*;
use k2f_core::LockFile;
use k2f_layout::LayoutEngine;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone)]
pub struct RunOptions {
    pub update_goldens: bool,
    pub dataset_runs: u32,
    pub check_png: bool,
    pub case_prefix: Option<String>,
}

pub fn run_suite(cases_dir: &Path, opts: &RunOptions) -> Result<()> {
    println!("Starting K2F Golden Test Suite...");
    println!("Cases Directory: {:?}", cases_dir);
    println!(
        "Options: update_goldens={}, dataset_runs={}, check_png={}",
        opts.update_goldens, opts.dataset_runs, opts.check_png
    );
    if let Some(prefix) = &opts.case_prefix {
        println!("Case filter: prefix='{}'", prefix);
    }

    if !cases_dir.exists() {
        anyhow::bail!("Test cases directory not found: {:?}", cases_dir);
    }

    let fallback_font = load_vendored_font()?;

    let mut failed_count = 0;
    let mut passed_count = 0;

    for entry in WalkDir::new(cases_dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
    {
        let case_path = entry.path();
        let case_name = case_path.file_name().unwrap().to_string_lossy();

        if let Some(prefix) = &opts.case_prefix {
            if !case_name.starts_with(prefix) {
                continue;
            }
        }

        match run_case(case_path, &fallback_font, opts) {
            Ok(CaseResult::Skipped) => {}
            Ok(CaseResult::Passed) => {
                passed_count += 1;
                println!("✅ {}", case_name.green());
            }
            Err(e) => {
                failed_count += 1;
                println!("❌ {} - {}", case_name.red(), e);
            }
        }
    }

    println!(
        "\nSUMMARY: {} Passed, {} Failed",
        passed_count, failed_count
    );
    if failed_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}

enum CaseResult {
    Passed,
    Skipped,
}

fn run_case(case_dir: &Path, fallback_font: &[u8], opts: &RunOptions) -> Result<CaseResult> {
    // 1. Load Inputs
    let content_path = case_dir.join("content.json");
    if !content_path.exists() {
        return Ok(CaseResult::Skipped);
    }

    let theme_path = case_dir.join("theme.json");
    let golden_lock_path = case_dir.join("golden.geometry.K2F.lock");
    let expected_error_path = case_dir.join("expected_error.txt");

    let content_str = fs::read_to_string(&content_path).context("missing content.json")?;
    let theme_str = fs::read_to_string(&theme_path).context("missing theme.json")?;
    let font_data = load_case_font(case_dir, fallback_font)?;
    let expected_error_substring = if expected_error_path.exists() {
        Some(fs::read_to_string(&expected_error_path)?.trim().to_string())
    } else {
        None
    };

    // 2. Compile (Determinism Check)
    let actual_lock_str = match compile_deterministic(
        case_dir,
        &content_str,
        &theme_str,
        &font_data,
        opts.dataset_runs,
    ) {
        Ok(lock) => {
            if expected_error_substring.is_some() {
                anyhow::bail!("Case compiled successfully but expected an error (expected_error.txt present).");
            }
            lock
        }
        Err(e) => {
            if let Some(substr) = expected_error_substring {
                // Expected failure case: pass if the error message contains the expected substring.
                if substr.is_empty() || e.to_string().contains(&substr) {
                    return Ok(CaseResult::Passed);
                }
                anyhow::bail!(
                    "Expected error containing '{substr}', but got: {actual}",
                    actual = e
                );
            }
            return Err(e);
        }
    };

    // 3. Compare with Golden Lock
    if opts.update_goldens {
        fs::write(&golden_lock_path, &actual_lock_str)?;
    } else {
        if !golden_lock_path.exists() {
            anyhow::bail!("Missing golden file. Run with --update to create it.");
        }

        let golden_str = fs::read_to_string(&golden_lock_path)?;
        let actual_lock: LockFile = parse_lockfile_unbounded(&actual_lock_str)?;
        let golden_lock: LockFile = parse_lockfile_unbounded(&golden_str)?;
        compare_golden_lock(&actual_lock, &golden_lock)?;
    }

    // 4. Optional PNG Verification
    if opts.check_png {
        let assets = load_case_assets(case_dir)?;
        verify_or_update_pngs(
            case_dir,
            &actual_lock_str,
            &font_data,
            &assets,
            opts.update_goldens,
        )?;
    }

    Ok(CaseResult::Passed)
}

fn compile_deterministic(
    case_dir: &Path,
    content_str: &str,
    theme_str: &str,
    font_data: &[u8],
    runs: u32,
) -> Result<String> {
    if runs == 0 {
        anyhow::bail!("dataset_runs must be >= 1");
    }

    let assets = load_case_assets(case_dir)?;

    let mut previous_lock: Option<String> = None;
    for i in 0..runs {
        let lock_json = if assets.is_empty() {
            LayoutEngine::compile_chunk(content_str, theme_str, font_data)
                .map_err(|e| anyhow::anyhow!("Compilation Error: {}", e))?
        } else {
            LayoutEngine::compile_chunk_with_assets(content_str, theme_str, font_data, &assets)
                .map_err(|e| anyhow::anyhow!("Compilation Error: {}", e))?
        };

        if let Some(ref prev) = previous_lock {
            if prev != &lock_json {
                anyhow::bail!("Determinism failure on run {}!", i + 1);
            }
        }
        previous_lock = Some(lock_json);
    }

    Ok(previous_lock.expect("runs>=1 ensured above"))
}

fn load_case_assets(case_dir: &Path) -> Result<HashMap<String, Vec<u8>>> {
    let assets_dir = case_dir.join("assets");
    if !assets_dir.exists() {
        return Ok(HashMap::new());
    }

    let mut out: HashMap<String, Vec<u8>> = HashMap::new();
    for entry in WalkDir::new(&assets_dir)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let abs = entry.path();
        let rel = abs
            .strip_prefix(case_dir)
            .map_err(|_| anyhow::anyhow!("Failed to compute relative asset path for {:?}", abs))?;
        let key = rel.to_string_lossy().replace('\\', "/");
        let bytes = fs::read(abs)?;
        out.insert(key, bytes);
    }

    Ok(out)
}

fn verify_or_update_pngs(
    case_dir: &Path,
    actual_lock_str: &str,
    font_data: &[u8],
    assets: &HashMap<String, Vec<u8>>,
    update: bool,
) -> Result<()> {
    let actual_lock: LockFile = parse_lockfile_unbounded(actual_lock_str)?;
    if !update && !case_dir.join("page_1.png").exists() {
        return Ok(());
    }
    let images: std::collections::BTreeMap<String, Vec<u8>> =
        assets.iter().map(|(k, v)| (k.clone(), v.clone())).collect();

    for (page_idx, _page) in actual_lock.geometry.pages.iter().enumerate() {
        let png_filename = format!("page_{}.png", page_idx + 1);
        let png_path = case_dir.join(&png_filename);
        let png_data = k2f_paint::render_lockfile_page_to_png(
            &actual_lock,
            page_idx,
            k2f_paint::OFFICIAL_PNG_SCALE,
            &k2f_paint::single_font_map(font_data),
            &images,
        )?;

        if update {
            fs::write(&png_path, &png_data)?;
            continue;
        }

        // Visual-gate cases commit page_*.png; other cases skip when absent.
        if !png_path.exists() {
            if page_idx == 0 {
                return Ok(());
            }
            anyhow::bail!(
                "Missing golden PNG: {}. Re-run with --update --check-png.",
                png_filename
            );
        }

        let stored_png = fs::read(&png_path)?;
        if stored_png != png_data {
            anyhow::bail!("Visual Mismatch! {} differs from golden.", png_filename);
        }
    }

    Ok(())
}

/// Compare layout output, not engine identity.
///
/// `engine_commit_sha` and `appearance_hash` (which includes that SHA) change
/// between debug (`UNKNOWN`) and release (git SHA). Schema-adjacent lock
/// metadata must not fail geometry goldens when pixels and glyph boxes match.
fn compare_golden_lock(actual: &LockFile, golden: &LockFile) -> Result<()> {
    if actual.content_hash != golden.content_hash {
        anyhow::bail!(
            "Content Hash Mismatch! Input semantic has changed but golden was not updated."
        );
    }
    if actual.geometry != golden.geometry {
        anyhow::bail!("Geometry Mismatch! Output differs from golden.");
    }
    if actual.render_plan != golden.render_plan {
        anyhow::bail!("Render plan mismatch! Output differs from golden.");
    }
    Ok(())
}

fn parse_lockfile_unbounded(json: &str) -> Result<LockFile> {
    let mut deserializer = serde_json::Deserializer::from_str(json);
    deserializer.disable_recursion_limit();
    let lock = LockFile::deserialize(&mut deserializer)?;
    Ok(lock)
}

fn load_vendored_font() -> Result<Vec<u8>> {
    let font_path: PathBuf = Path::new("assets/fonts/Roboto-Regular.ttf").to_path_buf();
    if !font_path.exists() {
        anyhow::bail!(
            "Font file not found at {:?}. Deterministic rendering requires embedded fonts.",
            font_path
        );
    }

    fs::read(&font_path).context("Failed to read font file")
}

fn load_case_font(case_dir: &Path, fallback: &[u8]) -> Result<Vec<u8>> {
    let hint = case_dir.join("font.txt");
    if !hint.exists() {
        return Ok(fallback.to_vec());
    }
    let name = fs::read_to_string(&hint)?.trim().to_string();
    if name.is_empty() || name.contains('/') || name.contains('\\') {
        anyhow::bail!("invalid font.txt in {:?}", case_dir);
    }
    let path = Path::new("assets/fonts").join(&name);
    fs::read(&path).with_context(|| format!("case font {path:?}"))
}
