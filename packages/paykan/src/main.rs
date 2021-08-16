mod config;
use futures::future::join_all;
use std::convert::Infallible;

use crate::config::Config;

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
    // let servers: Vec<_> = config
    //     .http
    //     .servers
    //     .into_iter()
    //     .map(|x| {
    //         let make_svc = make_service_fn({
    //             let server = x.clone();
    //             move |_conn| async {
    //                 Ok::<_, Infallible>(service_fn(|req| request_handler(req, &server)))
    //             }
    //         });
    //         let server = Server::bind(&x.listen).serve(make_svc);
    //         server
    //     })
    //     .collect();
    // let start_handlers = {
    //     let mut futures = vec![];
    //     for server in servers {
    //         futures.push(async {
    //             server.await.unwrap();
    //         })
    //     }
    //     futures
    // };
    // join_all(start_handlers).await;
    println!("Hello, world!");
}

// async fn request_handler(
//     req: Request<Body>,
//     server: &config::Server,
// ) -> Result<Response<Body>, Infallible> {

//     Ok(Response::new(
//         format!("Hello from {}.\n{:?}", server.server_name, req).into(),
//     ))
// }
