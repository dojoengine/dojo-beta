use starknet::core::types::FromStrError;
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider};

use crate::contract::component::ComponentError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error originated from the gRPC client.
    #[error(transparent)]
    GrpcClient(#[from] torii_grpc::client::Error),
    #[error(transparent)]
    FromStr(#[from] FromStrError),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
    #[error(transparent)]
    Component(#[from] ComponentError<<JsonRpcClient<HttpTransport> as Provider>::Error>),
}
