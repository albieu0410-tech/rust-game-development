use std::net::SocketAddr;

pub fn database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://deduced:deduced@localhost:5432/deduced".to_string())
}

pub fn bind_addr() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 4000))
}
