use crate::config::Server;

pub async fn serve(server: &Server) -> Result<(), ()> {
    println!("starting {}", server.listen);
    Ok(())
}
