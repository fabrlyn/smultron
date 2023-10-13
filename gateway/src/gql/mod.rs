use async_graphql::{EmptyMutation, Schema};

mod handler;
pub mod model;
mod query;
mod subscription;

pub use handler::Handler;

pub fn schema(handler: Box<dyn Handler>) -> Schema<query::Root, EmptyMutation, subscription::Root> {
    Schema::build(query::Root, EmptyMutation, subscription::Root)
        .data(handler)
        .finish()
}
