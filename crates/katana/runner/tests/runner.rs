use katana_runner::RunnerCtx;
use starknet::providers::Provider;

#[katana_runner::test(flavor = "binary", fee = false, accounts = 7)]
fn simple(runner: &RunnerCtx) {
    let runner = runner.as_binary().unwrap();
    assert_eq!(runner.accounts().len(), 7);
}

#[katana_runner::test]
fn with_return(_: &RunnerCtx) -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[katana_runner::test]
async fn with_async(ctx: &RunnerCtx) -> Result<(), Box<dyn std::error::Error>> {
    let ctx = ctx.as_binary().unwrap();
    let provider = ctx.provider();
    let _ = provider.chain_id().await?;
    Ok(())
}

#[should_panic]
#[katana_runner::test(
    flavor = "binary",
    fee = false,
    accounts = 7,
    block_time = 10,
    validation = true,
    program_path = "/path/to/binary"
)]
fn binary_args(_: &RunnerCtx) {}

#[should_panic]
#[katana_runner::test(
    flavor = "embedded",
    fee = false,
    accounts = 7,
    block_time = 10,
    validation = true
)]
#[tokio::test(flavor = "multi_thread")]
async fn embedded_runner(_: &RunnerCtx) {
    println!("Hello World!")
}
