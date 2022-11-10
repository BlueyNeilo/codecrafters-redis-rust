mod resp;
mod store;
mod server;

#[tokio::main]
async fn main() {
    tokio::join!(
        server::init(),
        server::listen()
    );
}
