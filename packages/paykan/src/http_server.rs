use std::error::Error;

///! HTTP1.1 based on https://datatracker.ietf.org/doc/html/rfc2616

use crate::config::Server;
use tokio::net::TcpListener;

pub async fn serve(server: &Server) -> Result<(), Box<dyn Error>> {
    println!("starting {}", server.listen);
    let listener = TcpListener::bind(server.listen).await?;
    loop {
        let (stream, addr) = listener.accept().await?;
    }
}


