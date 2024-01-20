use anyhow::Result;
use clap::Args;
use dojo_bindgen::{BuiltinPlugins, PluginManager};
use dojo_lang::scarb_internal::compile_workspace;
use scarb::core::{Config, TargetKind};
use scarb::ops::CompileOpts;

#[derive(Args, Debug)]
pub struct BuildArgs {
    #[arg(long)]
    #[arg(help = "Generate Typescript bindings.")]
    pub typescript: bool,

    #[arg(long)]
    #[arg(help = "Generate Unity bindings.")]
    pub unity: bool,
}

impl BuildArgs {
    pub fn run(self, config: &Config) -> Result<()> {
        let compile_info = compile_workspace(
            config,
            CompileOpts { include_targets: vec![], exclude_targets: vec![TargetKind::TEST] },
        )?;

        let mut builtin_plugins = vec![];
        if self.typescript {
            builtin_plugins.push(BuiltinPlugins::Typescript);
        }

        if self.unity {
            builtin_plugins.push(BuiltinPlugins::Unity);
        }

        // Custom plugins are always empty for now.
        let bindgen = PluginManager {
            artifacts_path: compile_info.target_dir,
            plugins: vec![],
            builtin_plugins,
        };

        tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(bindgen.generate())
            .expect("Error generating bindings");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use dojo_test_utils::compiler::build_test_config;

    use super::BuildArgs;

    // this adds considerable time to test suite, should this be enabled by default?
    #[ignore]
    #[test]
    fn build_example() {
        let config = build_test_config("../../examples/spawn-and-move/Scarb.toml").unwrap();

        let build_args = BuildArgs { typescript: false, unity: false };
        let result = build_args.run(&config);
        assert!(result.is_ok());
    }
}
