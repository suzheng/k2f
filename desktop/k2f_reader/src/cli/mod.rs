//! Binary entry: GUI unless `--export-pdf` / `--verify` (no window).

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
    #[arg(long, value_name = "OUT_PDF", requires = "file")]
    export_pdf: Option<PathBuf>,
    /// Print banner; exit non-zero if integrity is broken
    #[arg(long, requires = "file")]
    verify: bool,
}

pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    if cli.file.is_none() && cli.export_pdf.is_none() && !cli.verify {
        eprintln!(
            "usage: k2f-reader <file.K2F> | --verify <file.K2F> | --export-pdf out.pdf <file.K2F>"
        );
        std::process::exit(2);
    }
    let path = cli.file.as_deref().context("FILE")?;
    let app = open(path)?;
    if let Some(out) = cli.export_pdf.as_deref() {
        headless::export_pdf(&app, out)?;
    }
    if cli.verify {
        return headless::verify(&app);
    }
    if cli.export_pdf.is_some() {
        return Ok(());
    }
    ui::run(app, Some(path.to_path_buf()))
}

fn open(path: &Path) -> anyhow::Result<AppState> {
    AppState::open(&std::fs::read(path).with_context(|| format!("read {}", path.display()))?)
}
