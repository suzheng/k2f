use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "k2f-idml",
    about = "Experimental K2F lock → IDML exporter (not part of k2f CLI)",
    long_about = "Experimental one-way exporter from a locked .K2F package to an InDesign package.\n\n\
Does not modify the source package. IDML is not a K2F source (IDML_IS_NOT_A_SOURCE).\n\
This is not a second layout engine: page geometry comes from the published lock.\n\
The official entry is `k2f export-idml`; this crate binary is for converter development.\n\
Default output is a folder (`stem.idml` + `Document Fonts/`). Use `--idml-only` for a lone .idml.\n\
On failure, the IdmlError is printed to stderr and the process exits with code 1.",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Export a locked .K2F package to an InDesign package (does not modify the input)
    Export {
        #[arg(value_name = "PACKAGE")]
        package: PathBuf,
        #[arg(short = 'o', long = "output", value_name = "OUT")]
        output: PathBuf,
        /// Write a lone .idml (no Document Fonts folder).
        #[arg(long)]
        idml_only: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Export {
            package,
            output,
            idml_only,
        } => match std::fs::read(&package) {
            Ok(bytes) => match k2f_idml::export_handoff_bytes(&bytes) {
                Ok(pkg) => match k2f_idml::write_handoff_output(&pkg, &output, idml_only) {
                    Ok(_) => ExitCode::SUCCESS,
                    Err(e) => fail(&e),
                },
                Err(e) => fail(&e),
            },
            Err(e) => fail(&e),
        },
    }
}

fn fail(err: &dyn std::fmt::Display) -> ExitCode {
    eprintln!("{err}");
    ExitCode::from(1)
}
