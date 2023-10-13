mod config;
mod debugger;
mod device;
mod device_manager;
mod gateway;
mod gql;
mod gql_plug;
mod http_api;
mod hub;
mod ipso;
mod service;
mod service_finder;
mod timeout;

use std::error::Error;

use config::Config;

type AppResult = Result<(), Box<dyn Error + Send + Sync + 'static>>;

#[tokio::main]
async fn main() -> AppResult {
    tracing_subscriber::fmt().try_init()?;

    let config = Config {
        http_api: http_api::Config {
            port: 8000,
            address: [0, 0, 0, 0].into(),
        },
    };

    let gql_schema = gql::schema(Box::new(gql_plug::GqlPlug {}));

    http_api::run(&config.http_api, gql_schema).await
}
