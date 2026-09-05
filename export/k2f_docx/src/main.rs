use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "k2f-docx",
    about = "Experimental K2F lock → DOCX exporter (not part of k2f CLI)",
    long_about = "Experimental one-way exporter from a locked .K2F package to .docx.\n\n\
Does not modify the source package. DOCX is not a K2F source (DOCX_IS_NOT_A_SOURCE).\n\
This is not a second layout engine: page geometry comes from the published lock.\n\
Not wired into `k2f export-pdf` or the official k2f CLI.\n\
On failure, the error is printed to stderr and the process exits with code 1.",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Export a locked .K2F package to .docx (does not modify the input)
    Export {
        #[arg(value_name = "PACKAGE")]
        package: PathBuf,
        #[arg(short = 'o', long = "output", value_name = "OUT")]
        output: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.command {
        Commands::Export { package, output } => match std::fs::read(&package) {
            Ok(bytes) => match k2f_docx::export_bytes(&bytes) {
                Ok(docx) => match std::fs::write(&output, docx) {
                    Ok(()) => ExitCode::SUCCESS,
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
