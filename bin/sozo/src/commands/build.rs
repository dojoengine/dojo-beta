use anyhow::Result;
use clap::Args;
use dojo_bindgen::{BuiltinPlugins, PluginManager};
use dojo_lang::scarb_internal::compile_workspace;
use scarb::core::{Config, TargetKind};
use scarb::ops::CompileOpts;
use starknet::core::types::contract::SierraClass;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct BuildArgs {
    #[arg(long)]
    #[arg(help = "Generate Typescript bindings.")]
    pub typescript: bool,

    #[arg(long)]
    #[arg(help = "Generate Unity bindings.")]
    pub unity: bool,

    #[arg(long)]
    #[arg(help = "Output directory.", default_value = "bindings")]
    pub bindings_output: String,

    #[arg(long)]
    #[arg(help = "Report of all built contracts statistics")]
    pub stats: bool,
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

        if self.stats {
            let built_contract_paths: fs::ReadDir =
                fs::read_dir(compile_info.target_dir.as_str()).unwrap();

            println!("\n");
            for sierra_json_path in built_contract_paths {
                let sierra_json_path: PathBuf = sierra_json_path.unwrap().path();
                let filename = sierra_json_path.file_name().unwrap();
                println!(
                    "---------------Contract Stats for {}---------------\n",
                    filename.to_str().unwrap()
                );
                let file = File::open(sierra_json_path)?;
                let contract_artifact: SierraClass = serde_json::from_reader(&file)?;
                let contract_artifact = contract_artifact.flatten()?;

                println!(
                    "- Contract bytecode size (Number of felts in the program): {}",
                    contract_artifact.sierra_program.iter().count()
                );

                println!("- Contract Class size: {} bytes \n", file.metadata().unwrap().len());
            }
        }

        // Custom plugins are always empty for now.
        let bindgen = PluginManager {
            profile_name: compile_info.profile_name,
            output_path: self.bindings_output.into(),
            manifest_path: compile_info.manifest_path,
            root_package_name: compile_info
                .root_package_name
                .unwrap_or("NO_ROOT_PACKAGE".to_string()),
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

    #[test]
    fn build_example_with_typescript_and_unity_bindings() {
        let config = build_test_config("../../examples/spawn-and-move/Scarb.toml").unwrap();

        let build_args =
            BuildArgs { bindings_output: "generated".to_string(), typescript: true, unity: true };
        let result = build_args.run(&config);
        assert!(result.is_ok());
    }
}
