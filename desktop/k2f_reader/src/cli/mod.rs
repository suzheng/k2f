//! Binary entry: GUI unless `--export-pdf` / `--export-pptx` / `--export-docx` / `--verify` (no window).

mod headless;

use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::Parser;

use crate::ui;
use crate::AppState;

#[derive(Parser)]
#[command(name = "k2f-reader", about = "K2F desktop reader (lock executor)")]
struct Cli {
    /// Open a .K2F file in the GUI
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,
    /// Write PDF from lock to PATH (no window)
    #[arg(
        long,
        value_name = "OUT_PDF",
        requires = "file",
        conflicts_with_all = ["export_pptx", "export_docx"]
    )]
    export_pdf: Option<PathBuf>,
    /// Write PowerPoint .pptx from lock to PATH (no window)
    #[arg(
        long,
        value_name = "OUT_PPTX",
        requires = "file",
        conflicts_with_all = ["export_pdf", "export_docx"]
    )]
    export_pptx: Option<PathBuf>,
    /// Write Word .docx from lock to PATH (no window)
    #[arg(
        long,
        value_name = "OUT_DOCX",
        requires = "file",
        conflicts_with_all = ["export_pdf", "export_pptx"]
    )]
    export_docx: Option<PathBuf>,
    /// Print banner; exit non-zero if integrity is broken
    #[arg(long, requires = "file")]
    verify: bool,
}

/// True when this exe path is a packaged GUI install (not `cargo test` / CLI).
pub fn exe_looks_like_gui_launch(exe: &Path) -> bool {
    let s = exe.to_string_lossy().replace('\\', "/");
    let lower = s.to_ascii_lowercase();
    s.contains(".app/Contents/MacOS/")
        || lower.ends_with("/k2f reader/k2f-reader.exe")
        || lower.ends_with("/k2f-reader.appimage")
}

fn in_cargo_or_ci() -> bool {
    std::env::var_os("CARGO").is_some() || std::env::var_os("CI").is_some()
}

#[cfg(windows)]
fn windows_double_clicked() -> bool {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetConsoleProcessList(process_list: *mut u32, count: u32) -> u32;
    }
    let mut buf = [0u32; 4];
    let n = unsafe { GetConsoleProcessList(buf.as_mut_ptr(), buf.len() as u32) };
    n == 1
}

fn should_prompt_open() -> bool {
    if in_cargo_or_ci() {
        return false;
    }
    if std::env::var_os("APPIMAGE").is_some() || std::env::var_os("K2F_READER_GUI").is_some() {
        return true;
    }
    if std::env::current_exe()
        .ok()
        .is_some_and(|exe| exe_looks_like_gui_launch(&exe))
    {
        return true;
    }
    #[cfg(windows)]
    {
        if windows_double_clicked() {
            return true;
        }
    }
    false
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if cli.file.is_none()
        && cli.export_pdf.is_none()
        && cli.export_pptx.is_none()
        && cli.export_docx.is_none()
        && !cli.verify
    {
        if should_prompt_open() {
            let Some(path) = crate::export::pick_open_path() else {
                return Ok(());
            };
            let app = open(&path)?;
            return ui::run(app, Some(path));
        }
        eprintln!(
            "usage: k2f-reader <file.K2F> | --verify <file.K2F> | --export-pdf out.pdf <file.K2F> | --export-pptx out.pptx <file.K2F> | --export-docx out.docx <file.K2F>"
        );
        std::process::exit(2);
    }
    let path = cli.file.as_deref().context("FILE")?;
    let app = open(path)?;
    if let Some(out) = cli.export_pdf.as_deref() {
        headless::export_pdf(&app, out)?;
    }
    if let Some(out) = cli.export_pptx.as_deref() {
        headless::export_pptx(&app, out)?;
    }
    if let Some(out) = cli.export_docx.as_deref() {
        headless::export_docx(&app, out)?;
    }
    if cli.verify {
        return headless::verify(&app);
    }
    if cli.export_pdf.is_some() || cli.export_pptx.is_some() || cli.export_docx.is_some() {
        return Ok(());
    }
    ui::run(app, Some(path.to_path_buf()))
}

fn open(path: &Path) -> anyhow::Result<AppState> {
    AppState::open(&std::fs::read(path).with_context(|| format!("read {}", path.display()))?)
}
