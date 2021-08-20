pub mod config;
pub mod http_server;
pub mod lazy_stream_reader;

use crate::config::Config;
use futures::future::join_all;

#[tokio::main]
async fn main() {
    let config = Config::from(parser::parse(
        r#"
    http {
        server {
            server_name "server_name";
            listen 127.0.0.1:8080;
        }
        server {
            server_name "server_name2";
            listen 127.0.0.1:8081;
        }
    }
    "#,
    ));
    let servers = config
        .http
        .servers
        .iter()
        .map(|server| http_server::serve(server));

    join_all(servers).await;
}
