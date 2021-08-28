use std::error::Error;

///! HTTP1.1 based on https://datatracker.ietf.org/doc/html/rfc2616
use crate::config::Server;
use crate::lazy_stream_reader::HttpLazyStreamReader;
use tokio::net::TcpListener;

pub async fn serve(server: &Server) -> Result<(), Box<dyn Error>> {
    println!("starting {}", server.listen);
    let listener = TcpListener::bind(server.listen).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let reader = HttpLazyStreamReader::new(Box::pin(stream));
        println!("method: {:?}", reader.method().await);
    }
}
