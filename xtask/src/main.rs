mod release;
mod wasm;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "xtask", about = "brepkit build automation")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Build WASM package with dual targets, merge, and validate.
    WasmBuild {
        /// Disable SIMD optimizations (simd128 is enabled by default).
        #[arg(long)]
        no_simd: bool,

        /// Skip wasm-opt optimization pass.
        #[arg(long)]
        skip_opt: bool,
    },

    /// Build, validate, and publish WASM package to npm.
    WasmPublish {
        /// Run npm publish with --dry-run.
        #[arg(long)]
        dry_run: bool,

        /// Disable SIMD optimizations (simd128 is enabled by default).
        #[arg(long)]
        no_simd: bool,
    },

    /// Compute the next version from git history, bump the crate, and publish.
    WasmRelease {
        /// Run npm publish with --dry-run.
        #[arg(long)]
        dry_run: bool,

        /// Disable SIMD optimizations (simd128 is enabled by default).
        #[arg(long)]
        no_simd: bool,

        /// Allow a breaking change to bump the major version. Default clamps
        /// major to minor because the fork inherits upstream's version line.
        #[arg(long)]
        allow_major: bool,

        /// Override the computed version with an explicit X.Y.Z.
        #[arg(long)]
        version: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::WasmBuild { no_simd, skip_opt } => {
            wasm::check_tools()?;
            wasm::build_both_targets(!no_simd)?;
            if !skip_opt {
                wasm::run_wasm_opt()?;
            }
            wasm::merge_packages()?;
            wasm::validate_output()?;
            println!("\n✅ WASM build complete. Run smoke test with:");
            println!("   node scripts/test-wasm-smoke.mjs");
        }
        Command::WasmPublish { dry_run, no_simd } => {
            wasm::check_tools()?;
            wasm::build_both_targets(!no_simd)?;
            wasm::run_wasm_opt()?;
            wasm::merge_packages()?;
            wasm::validate_output()?;
            wasm::run_smoke_test()?;
            wasm::publish(dry_run)?;
        }
        Command::WasmRelease {
            dry_run,
            no_simd,
            allow_major,
            version,
        } => {
            wasm::check_tools()?;
            release::run_release(wasm::RunReleaseOpts {
                dry_run,
                simd: !no_simd,
                allow_major,
                version,
            })?;
        }
    }

    Ok(())
}
