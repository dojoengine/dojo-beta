#![cfg_attr(not(test), warn(unused_crate_dependencies))]

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use hyper::{Method, Uri};
use jsonrpsee::core::{async_trait, RpcResult};
use jsonrpsee::server::{middleware::http::ProxyGetRequestLayer, ServerBuilder};
use katana_rpc_api::katana::KatanaApiServer;
use katana_rpc_types::account::Account;
use tower::Layer;
use tower_http::cors::{AllowOrigin, CorsLayer};

pub struct HttpServer;

#[async_trait]
impl KatanaApiServer for HttpServer {
    async fn predeployed_accounts(&self) -> RpcResult<Vec<Account>> {
        println!("http");
        Ok(Vec::new())
    }
}

pub struct WsServer;

#[async_trait]
impl KatanaApiServer for WsServer {
    async fn predeployed_accounts(&self) -> RpcResult<Vec<Account>> {
        println!("ws");
        Ok(Vec::new())
    }
}

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new().allow_methods([Method::POST, Method::GET]);

    let middleware = tower::ServiceBuilder::new().option_layer(Some(cors.clone()));
    let module1 = HttpServer.into_rpc();
    let http = ServerBuilder::new().http_only().set_http_middleware(middleware);

    let module2 = WsServer.into_rpc();
    let middleware = tower::ServiceBuilder::new().option_layer(Some(cors));
    let ws = ServerBuilder::new().ws_only().set_http_middleware(middleware);

    let http = http.build("localhost:5050").await.unwrap();

    let addr = http.local_addr().unwrap();
    let handle = http.start(module1);
    println!("http server started: {}", addr);

    let ws = ws.build("localhost:6969").await.unwrap();
    let addr = ws.local_addr().unwrap();
    let handle = ws.start(module2);
    println!("ws server started: {}", addr);

    futures::future::pending::<()>().await;
}
