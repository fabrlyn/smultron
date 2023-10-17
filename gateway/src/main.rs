mod application;
mod config;
mod domain;
mod infrastructure;
mod interface;

use std::error::Error;

use config::Config;
use interface::{adapter::InMemoryAdapter, gql, http};

type AppResult = Result<(), Box<dyn Error + Send + Sync + 'static>>;

#[tokio::main]
async fn main() -> AppResult {
    tracing_subscriber::fmt().try_init()?;

    let config = Config {
        http_api: http::Config {
            port: 8000,
            address: [0, 0, 0, 0].into(),
        },
    };

    let gql_schema = gql::schema(Box::new(InMemoryAdapter {}));

    http::run(&config.http_api, gql_schema).await
}
