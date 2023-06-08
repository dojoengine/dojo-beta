use clap::Parser;
use dojo_world::manifest::Manifest;
use graphql::server::start_graphql;
use sqlx::sqlite::SqlitePoolOptions;
use starknet::core::types::FieldElement;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::JsonRpcClient;
use state::sql::Sql;
use tokio_util::sync::CancellationToken;
use tracing::error;
use tracing_subscriber::fmt;
use url::Url;

use crate::engine::Processors;
use crate::indexer::Indexer;

mod engine;
mod graphql;
mod indexer;
mod processors;
mod state;
mod types;

#[cfg(test)]
mod tests;

/// Dojo World Indexer
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The world to index
    #[arg(short, long, default_value = "0x420")]
    world_address: FieldElement,
    /// The rpc endpoint to use
    #[arg(long, default_value = "http://localhost:5050")]
    rpc: String,
    /// Database url
    #[arg(short, long, default_value = "sqlite::memory:")]
    database_url: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let subscriber = fmt::Subscriber::builder()
        .with_max_level(tracing::Level::INFO) // Set the maximum log level
        .finish();

    // Set the global subscriber
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set the global tracing subscriber");

    // Setup cancellation for graceful shutdown
    let cts = CancellationToken::new();
    ctrlc::set_handler({
        let cts: CancellationToken = cts.clone();
        move || {
            cts.cancel();
        }
    })?;

    let database_url = &args.database_url;
    #[cfg(feature = "sqlite")]
    let pool = SqlitePoolOptions::new().max_connections(5).connect(database_url).await?;
    let provider = JsonRpcClient::new(HttpTransport::new(Url::parse(&args.rpc).unwrap()));

    let manifest = Manifest::default();
    let state = Sql::new(pool.clone(), args.world_address).await?;
    let indexer = Indexer::new(&state, &provider, Processors::default(), manifest);
    let graphql = start_graphql(&pool);

    tokio::select! {
        res = indexer.start() => {
            if let Err(e) = res {
                error!("Indexer failed with error: {:?}", e);
            }
        }
        res = graphql => {
            if let Err(e) = res {
                error!("GraphQL server failed with error: {:?}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Received Ctrl+C, shutting down");
        }
    }

    Ok(())
}
