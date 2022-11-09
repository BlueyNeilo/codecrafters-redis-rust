#[allow(unused_imports)]
use std::env;
#[allow(unused_imports)]
use std::fs;

mod resp;
mod store;
mod server;

#[tokio::main]
async fn main() {
    server::init().await;
    server::listen().await;
}
