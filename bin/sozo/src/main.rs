use std::env;
use std::process::exit;

use anyhow::Result;
use args::SozoArgs;
use clap::Parser;
use dojo_lang::compiler::DojoCompiler;
use dojo_lang::plugin::CairoPluginRepository;
use scarb::compiler::CompilerRepository;
use scarb::core::Config;
use scarb_ui::{OutputFormat, Ui};
use tracing::trace;
use crate::commands::Commands;

mod args;
mod commands;
mod utils;

pub(crate) const LOG_TARGET: &str = "sozo::cli";
fn main() {
    
    let args = SozoArgs::parse();
    let _ = args.init_logging();
    trace!(target: LOG_TARGET, command = ?args.command, "Sozo CLI command started.");
    let ui = Ui::new(args.ui_verbosity(), OutputFormat::Text);

    if let Err(err) = cli_main(args) {
        ui.anyhow(&err);
        exit(1);
    }
}

fn cli_main(args: SozoArgs) -> Result<()> {
    let mut compilers = CompilerRepository::std();
    let cairo_plugins = CairoPluginRepository::default();

    match &args.command {
        Commands::Build(_) | Commands::Dev(_) | Commands::Migrate(_) => {
            trace!(target: LOG_TARGET, "Adding DojoCompiler to compiler repository");
            compilers.add(Box::new(DojoCompiler)).unwrap()
        }
        _ => {
            trace!(target: LOG_TARGET, "No additional compiler required");
        }
    }

    let manifest_path = scarb::ops::find_manifest_path(args.manifest_path.as_deref())?;

    utils::verify_cairo_version_compatibility(&manifest_path)?;

    let config = Config::builder(manifest_path)
        .log_filter_directive(env::var_os("SCARB_LOG"))
        .profile(args.profile_spec.determine()?)
        .offline(args.offline)
        .cairo_plugins(cairo_plugins.into())
        .ui_verbosity(args.ui_verbosity())
        .compilers(compilers)
        .build()?;

    trace!(target: LOG_TARGET, "Configuration built successfully");

    let _ = commands::run(args.command, &config);

    trace!(target: LOG_TARGET, "Command execution completed");

    Ok(())
}