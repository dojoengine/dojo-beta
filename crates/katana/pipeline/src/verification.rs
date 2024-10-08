use anyhow::Result;
use cairo_lang_runner::{Arg, ProfilingInfoCollectionConfig, RunResultValue, SierraCasmRunner};
use cairo_lang_sierra::program::VersionedProgram;
use cairo_lang_utils::ordered_hash_map::OrderedHashMap;
use cairo_verifier_runner::transform::StarkProofExprs;
use cairo_verifier_runner::vec252::VecFelt252;
use itertools::{chain, Itertools};
use katana_primitives::Felt;
use swiftness_proof_parser::parse;

// the raw proof json string
fn verify_state_diff(proof: String) -> Result<()> {
    let stark_proof: StarkProofExprs = parse(proof)?.into();

    let config = serde_json::from_str::<VecFelt252>(&stark_proof.config.to_string())?;
    let public_input = serde_json::from_str::<VecFelt252>(&stark_proof.public_input.to_string())?;
    let unsent_commitment =
        serde_json::from_str::<VecFelt252>(&stark_proof.unsent_commitment.to_string())?;
    let witness = serde_json::from_str::<VecFelt252>(&stark_proof.witness.to_string())?;

    let proof = chain!(
        config.into_iter(),
        public_input.into_iter(),
        unsent_commitment.into_iter(),
        witness.into_iter()
    )
    .collect_vec();

    let contract = include_str!("../../contracts/compiled/cairo_verifier.json");
    let sierra = serde_json::from_str::<VersionedProgram>(&contract)?.into_v1()?;

    let runner = SierraCasmRunner::new(
        sierra.program,
        Some(Default::default()),
        OrderedHashMap::default(),
        Some(ProfilingInfoCollectionConfig::default()),
    )?;

    // TODO: update cairo-verifier to use latest cairo version
    let cairo_version = Felt::ONE;
    // convert to new felt type
    let proof =
        proof.into_iter().map(|f| Arg::Value(Felt::from_bytes_be(&f.to_be_bytes()))).collect_vec();
    let args = &[Arg::Array(proof), Arg::Value(cairo_version)];

    // main function entrypoint
    let func = runner.find_function("main")?;
    let result = runner
        .run_function_with_starknet_context(func, args, Some(u32::MAX as usize), Default::default())
        .unwrap();

    match result.value {
        RunResultValue::Success(msg) => {
            println!("{:?}", msg);
        }
        RunResultValue::Panic(msg) => {
            panic!("{:?}", msg);
        }
    }

    Ok(())
}
