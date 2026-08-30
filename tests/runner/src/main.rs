use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod node_builders;
mod test_cases;
mod themes;

mod generator;
use generator::Generator;

mod suite;
use suite::{run_suite, RunOptions};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the test suite against all cases in the directory
    Run {
        #[arg(short, long, default_value = "tests/fixtures/cases")]
        cases_dir: PathBuf,

        /// Update golden files if they mismatch
        #[arg(long)]
        update: bool,

        /// Number of times to run each case for determinism check.
        ///
        /// If omitted, defaults to 5 in CI and 1 locally.
        #[arg(long)]
        dataset_runs: Option<u32>,

        /// Also verify (or update) per-page PNG golden artifacts.
        ///
        /// Lock JSON is the primary determinism gate; PNGs are an optional stronger check.
        #[arg(long)]
        check_png: bool,

        /// Only run cases whose directory name starts with this prefix (e.g. "semantic_code_blocks_").
        #[arg(long)]
        case_prefix: Option<String>,
    },

    /// Generate synthetic test cases for the suite
    Generate {
        #[arg(short, long, default_value = "tests/fixtures/cases")]
        output_dir: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            cases_dir,
            update,
            dataset_runs,
            check_png,
            case_prefix,
        } => {
            let runs = dataset_runs.unwrap_or_else(default_dataset_runs);
            if runs == 0 {
                anyhow::bail!("--dataset-runs must be >= 1");
            }

            run_suite(
                &cases_dir,
                &RunOptions {
                    update_goldens: update,
                    dataset_runs: runs,
                    check_png,
                    case_prefix,
                },
            )?;
        }
        Commands::Generate { output_dir } => {
            Generator::generate_all(&output_dir)?;
        }
    }

    Ok(())
}

fn default_dataset_runs() -> u32 {
    // In CI we default to 5 runs to catch determinism issues.
    // Locally we default to 1 run to keep the suite quick.
    //
    // We detect CI via the conventional `CI` environment variable.
    if std::env::var_os("CI").is_some() {
        5
    } else {
        1
    }
}
