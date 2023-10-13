use std::net::IpAddr;
use std::net::SocketAddr;

use async_graphql::http::GraphiQLSource;
use async_graphql::Executor;
use async_graphql_axum::{GraphQL, GraphQLSubscription};
use axum::{
    response::{self, IntoResponse},
    routing::get,
    Router, Server,
};

use crate::AppResult;

const GQL: &str = "/gql";
const GQL_SUBSCRIPTION: &str = "/gql/subscription";

#[derive(Clone, Debug)]
pub struct Config {
    pub port: u16,
    pub address: IpAddr,
}

impl Config {
    pub fn socket_address(&self) -> SocketAddr {
        SocketAddr::new(self.address, self.port)
    }
}

async fn gql_ui() -> impl IntoResponse {
    response::Html(
        GraphiQLSource::build()
            .endpoint(GQL)
            .subscription_endpoint(GQL_SUBSCRIPTION)
            .finish(),
    )
}

fn gql_router<S>(schema: S) -> Router
where
    S: Executor,
{
    Router::new()
        .route(GQL, get(gql_ui).post_service(GraphQL::new(schema.clone())))
        .route_service(GQL_SUBSCRIPTION, GraphQLSubscription::new(schema))
}

pub async fn run<S>(config: &Config, schema: S) -> AppResult
where
    S: Executor,
{
    Server::bind(&config.socket_address())
        .serve(gql_router(schema).into_make_service())
        .await
        .map_err(Into::into)
}
