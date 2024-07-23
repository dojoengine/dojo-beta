use std::env;
use std::path::PathBuf;

use prover_sdk::{Cairo0ProverInput, Cairo1ProverInput};
use serde_json::Value;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

use super::{ProveDiffProgram, ProveProgram};

pub async fn load_program(prove_program: ProveProgram) -> anyhow::Result<Value> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let program_file = match prove_program {
        ProveProgram::DiffProgram(ProveDiffProgram::Differ) => {
            manifest_dir.join("programs/cairo0differ.json")
        }
        ProveProgram::DiffProgram(ProveDiffProgram::Merger) => {
            manifest_dir.join("programs/cairo0merger.json")
        }
        ProveProgram::Checker => manifest_dir.join("programs/cairo1checker.json"),
        ProveProgram::Batcher => manifest_dir.join("programs/cairo1batcher.json"),
    };
    let mut program_file = File::open(program_file).await?;

    let mut data = String::new();
    program_file.read_to_string(&mut data).await?;
    let json_value: Value = serde_json::from_str(&data)?;

    Ok(json_value)
}

pub async fn prepare_input_cairo0(
    arguments: String,
    prove_program: ProveProgram,
) -> anyhow::Result<Cairo0ProverInput> {
    let program = load_program(prove_program).await?;

    let program = serde_json::from_str(&serde_json::to_string(&program)?)?;
    let program_input: Value = serde_json::from_str(&arguments)?;

    Ok(Cairo0ProverInput { program, program_input, layout: "recursive".into() })
}

pub async fn prepare_input_cairo1(
    arguments: String,
    prove_program: ProveProgram,
) -> anyhow::Result<Cairo1ProverInput> {
    let mut program = load_program(prove_program).await?;

    if let Value::Object(ref mut obj) = program {
        obj.insert("version".to_string(), Value::Number(serde_json::Number::from(1)));
    }

    let program = serde_json::from_str(&serde_json::to_string(&program)?)?;

    let program_input = Value::Array(vec![Value::String(arguments)]);
    Ok(Cairo1ProverInput { program, program_input, layout: "recursive".into() })
}
