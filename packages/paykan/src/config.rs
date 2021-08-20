use parser::Block;
use std::{net::SocketAddr, str::FromStr};

pub struct Config {
    pub http: Http,
}

#[derive(Debug, Clone)]
pub struct Http {
    pub servers: Vec<Server>,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub server_name: String,
    pub listen: SocketAddr,
}

impl From<Block> for Config {
    fn from(b: Block) -> Self {
        let http = b.directives.iter().find(|x| x.name == "http").unwrap();
        let servers: Vec<_> = http
            .block
            .as_ref()
            .unwrap()
            .directives
            .iter()
            .filter(|x| x.name == "server")
            .map(|x| x.block.as_ref().unwrap())
            .map(|x| Server {
                server_name: x
                    .directives
                    .iter()
                    .find(|y| y.name == "server_name")
                    .unwrap()
                    .parameters
                    .first()
                    .unwrap()
                    .1
                    .clone(),
                listen: SocketAddr::from_str(
                    &x.directives
                        .iter()
                        .find(|y| y.name == "listen")
                        .unwrap()
                        .parameters
                        .first()
                        .unwrap()
                        .1,
                )
                .unwrap(),
            })
            .collect();

        Self {
            http: Http { servers },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parser::parse;
    use std::net::SocketAddr;

    #[test]
    fn test_config() {
        let config = parse(
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
        );
        let conf = Config::from(config);
        assert_eq!(conf.http.servers.len(), 2);
        assert_eq!(conf.http.servers[0].server_name, "server_name");
        assert_eq!(
            conf.http.servers[0].listen,
            SocketAddr::from_str("127.0.0.1:8080").unwrap()
        );
        assert_eq!(conf.http.servers[1].server_name, "server_name2");
        assert_eq!(
            conf.http.servers[1].listen,
            SocketAddr::from_str("127.0.0.1:8081").unwrap()
        );
    }
}
