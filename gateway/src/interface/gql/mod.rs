use async_graphql::{EmptyMutation, Schema};

use crate::application::port::Port;

pub mod model;
mod query;
mod subscription;

pub fn schema(port: Box<dyn Port>) -> Schema<query::Root, EmptyMutation, subscription::Root> {
    Schema::build(query::Root, EmptyMutation, subscription::Root)
        .data(port)
        .finish()
}
