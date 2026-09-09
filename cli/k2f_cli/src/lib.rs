mod markdown;

use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use k2f_layout::compile_outcome;
use k2f_package::{
    bundled_schema_files, generate_secret_key, inspect_package, load_dir, pack_bytes, sign_package,
    unpack_bytes, utc_unix_seconds, write_dir, SecretKey, WriteDirOpts,
};
use k2f_paint::{OpenedDocument, OFFICIAL_PNG_SCALE};
use k2f_pdf::{export_opened, PdfExportOptions, PdfScale, DEFAULT_EXPORT_SCALE};

use markdown::{collect_md, convert_markdown};

#[derive(Parser)]
#[command(
    name = "k2f",
    author,
    version,
    about = "K2F pack / compile / verify / sign / render / hit-test"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Pack {
        source: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Unpack a .K2F package to a source directory.
    Unpack {
        package: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        /// Include document.K2F.lock (product round-trip).
        #[arg(long)]
        include_lock: bool,
    },
    Compile {
        package: PathBuf,
    },
    Verify {
        package: PathBuf,
    },
    Keygen {
        #[arg(short, long)]
        output: PathBuf,
    },
    Sign {
        package: PathBuf,
        #[arg(long)]
        key: PathBuf,
        #[arg(long)]
        signed_by: Option<String>,
        #[arg(long)]
        signed_at: Option<i64>,
    },
    Render {
        package: PathBuf,
        #[arg(long, default_value_t = 0)]
        page: usize,
        #[arg(long, default_value_t = OFFICIAL_PNG_SCALE)]
        scale: f32,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Hit-test lock geometry. x/y are pt, not pixels.
    HitTest {
        package: PathBuf,
        #[arg(long, default_value_t = 0)]
        page: usize,
        #[arg(long)]
        x: f64,
        #[arg(long)]
        y: f64,
    },
    /// Draw the published lock into a PDF. Not a second layout engine.
    ExportPdf {
        package: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long, default_value_t = DEFAULT_EXPORT_SCALE)]
        scale: f32,
        /// Append per-page source captions and a final integrity verification page.
        #[arg(long)]
        trust_pack: bool,
    },
    /// Draw the published lock into a PowerPoint .pptx. Not a second layout engine.
    ExportPptx {
        package: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Draw the published lock into a Word .docx. Not a second layout engine.
    ExportDocx {
        package: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Compile Markdown to a .K2F package. `--template` is an author directory.
    Markdown {
        source: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
        #[arg(long)]
        template: PathBuf,
        /// Optional OTF/TTF embedded instead of the template default face.
        #[arg(long)]
        font: Option<PathBuf>,
    },
    /// Dump embedded format JSON Schemas (for version reconciliation).
    Schema {
        #[command(subcommand)]
        command: SchemaCommands,
    },
}

#[derive(Subcommand)]
enum SchemaCommands {
    /// Write the five format schema files to a directory.
    Dump {
        #[arg(short, long)]
        output: PathBuf,
    },
}

pub fn run() -> anyhow::Result<()> {
    run_from(std::env::args())
}

pub fn run_from<I, T>(args: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    dispatch(Cli::parse_from(args).command)
}

fn dispatch(command: Commands) -> anyhow::Result<()> {
    match command {
        Commands::Pack { source, output } => {
            let pkg = load_dir(&source)?;
            fs::write(&output, pack_bytes(&pkg)?)?;
            eprintln!("packed {}", output.display());
        }
        Commands::Unpack {
            package,
            output,
            include_lock,
        } => {
            let pkg = unpack_bytes(&fs::read(&package)?)?;
            write_dir(
                &pkg,
                &output,
                WriteDirOpts {
                    include_lock,
                    include_schema: false,
                },
            )?;
            eprintln!("unpacked {}", output.display());
        }
        Commands::Compile { package } => {
            let mut pkg = unpack_bytes(&fs::read(&package)?)?;
            k2f_package::apply_coverage_subset(&mut pkg.fonts)?;
            let assets: std::collections::HashMap<String, Vec<u8>> =
                pkg.assets.clone().into_iter().collect();
            let outcome = compile_outcome(
                pkg.engine_manifest(),
                &pkg.theme_json,
                &pkg.fonts,
                if assets.is_empty() {
                    None
                } else {
                    Some(&assets)
                },
            )
            .map_err(|e| anyhow::anyhow!(e))?;
            pkg.set_lock(&outcome.lock)?;
            fs::write(&package, pack_bytes(&pkg)?)?;
            eprintln!(
                "compiled {} pages={}",
                package.display(),
                outcome.lock.geometry.pages.len()
            );
            for diag in &outcome.diags {
                eprintln!("{diag}");
            }
        }
        Commands::Verify { package } => {
            let bytes = fs::read(&package)?;
            let pkg = unpack_bytes(&bytes)?;
            let report = inspect_package(&pkg)?;
            println!("{}", report.status.code());
            if pkg.lock_json.is_some() {
                if let Ok(doc) = OpenedDocument::open(&bytes) {
                    eprintln!("pages={}", doc.page_count());
                }
            }
            if let Some(fp) = &report.fingerprint {
                eprintln!(
                    "fingerprint={fp} signed_by={} signed_at={} generated_by={}",
                    report.signed_by.as_deref().unwrap_or("-"),
                    report
                        .signed_at
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| "-".into()),
                    report.generated_by.as_deref().unwrap_or("-")
                );
            } else if let Some(agent) = &report.generated_by {
                eprintln!("generated_by={agent}");
            }
            if !report.status.is_ok() {
                std::process::exit(1);
            }
        }
        Commands::Keygen { output } => {
            let key = generate_secret_key()?;
            fs::write(&output, format!("{}\n", key.to_hex()))?;
            eprintln!("public={}", key.public_hex());
            eprintln!("fingerprint={}", key.fingerprint());
        }
        Commands::Sign {
            package,
            key,
            signed_by,
            signed_at,
        } => {
            let mut pkg = unpack_bytes(&fs::read(&package)?)?;
            let secret = SecretKey::from_hex(fs::read_to_string(&key)?.trim())?;
            sign_package(
                &mut pkg,
                &secret,
                signed_by.as_deref(),
                signed_at.unwrap_or_else(utc_unix_seconds),
            )?;
            fs::write(&package, pack_bytes(&pkg)?)?;
            eprintln!(
                "signed {} fingerprint={}",
                package.display(),
                secret.fingerprint()
            );
        }
        Commands::Render {
            package,
            page,
            scale,
            output,
        } => {
            let doc = OpenedDocument::open(&fs::read(&package)?)?;
            println!("{}", doc.banner().as_str());
            if let Some(fp) = doc.fingerprint() {
                eprintln!(
                    "fingerprint={fp} signed_by={} generated_by={}",
                    doc.signed_by().unwrap_or("-"),
                    doc.generated_by().unwrap_or("-")
                );
            }
            if doc.banner() != k2f_paint::Banner::Signed
                && doc.banner() != k2f_paint::Banner::Unsigned
            {
                eprintln!("{}", doc.status_code());
            }
            let n = doc.page_count();
            eprintln!("rendering page {page} of {n}");
            let png = doc.render_page(page, scale)?;
            fs::write(&output, png)?;
            if !matches!(
                doc.banner(),
                k2f_paint::Banner::Signed | k2f_paint::Banner::Unsigned
            ) {
                std::process::exit(1);
            }
        }
        Commands::HitTest {
            package,
            page,
            x,
            y,
        } => {
            let doc = OpenedDocument::open(&fs::read(&package)?)?;
            let milli = |pt: f64| (pt * 1000.0).round() as i64;
            match doc.hit_test(page, milli(x), milli(y)) {
                Some(hit) => {
                    println!("{}", hit.ids.join(" "));
                    if let Some([a, b]) = hit.char_range {
                        eprintln!("chars {a}..{b}");
                    }
                }
                None => {
                    println!("MISS");
                    std::process::exit(1);
                }
            }
        }
        Commands::ExportPdf {
            package,
            output,
            scale,
            trust_pack,
        } => {
            let scale = PdfScale::from_f32(scale).map_err(|e| anyhow::anyhow!(e))?;
            let doc = OpenedDocument::open(&fs::read(&package)?)?;
            let mut options = PdfExportOptions::new(scale);
            if trust_pack {
                options = options.with_trust_pack();
            }
            let pdf = export_opened(&doc, options)?;
            fs::write(&output, pdf)?;
            eprintln!("wrote {}", output.display());
        }
        Commands::ExportPptx { package, output } => {
            let bytes = k2f_pptx::export_bytes(&fs::read(&package)?)?;
            fs::write(&output, bytes)?;
            eprintln!("wrote {}", output.display());
        }
        Commands::ExportDocx { package, output } => {
            let bytes = k2f_docx::export_bytes(&fs::read(&package)?)?;
            fs::write(&output, bytes)?;
            eprintln!("wrote {}", output.display());
        }
        Commands::Markdown {
            source,
            output,
            template,
            font,
        } => {
            let font_bytes = match font {
                Some(path) => Some(fs::read(&path)?),
                None => None,
            };
            if source.is_dir() {
                let mut files = Vec::new();
                collect_md(&source, &mut files)?;
                files.sort();
                if files.is_empty() {
                    anyhow::bail!("no .md files under {}", source.display());
                }
                for md_path in files {
                    let rel = md_path.strip_prefix(&source).unwrap_or(&md_path);
                    let dest = output.join(rel).with_extension("K2F");
                    convert_markdown(&md_path, &dest, &template, font_bytes.as_deref())?;
                    eprintln!("wrote {}", dest.display());
                }
            } else {
                convert_markdown(&source, &output, &template, font_bytes.as_deref())?;
                eprintln!("wrote {}", output.display());
            }
        }
        Commands::Schema { command } => match command {
            SchemaCommands::Dump { output } => {
                fs::create_dir_all(&output)?;
                for (rel, text) in bundled_schema_files() {
                    let name = rel.rsplit('/').next().unwrap_or(rel);
                    let path = output.join(name);
                    fs::write(&path, text)?;
                    eprintln!("wrote {}", path.display());
                }
            }
        },
    }
    Ok(())
}
