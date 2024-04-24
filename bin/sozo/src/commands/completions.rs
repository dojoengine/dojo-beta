use std::io;

use anyhow::Result;
use clap::{Args, CommandFactory};
use clap_complete::{generate, Shell};
use tracing::trace;

use crate::args::SozoArgs;

pub(crate) const LOG_TARGET: &str = "sozo::cli::commands::completions";

#[derive(Debug, Args)]
pub struct CompletionsArgs {
    shell: Shell,
}

impl CompletionsArgs {
    pub fn run(self) -> Result<()> {
        let mut command = SozoArgs::command();
        let name = command.get_name().to_string();
        generate(self.shell, &mut command, name, &mut io::stdout());
        Ok(())
    }
}