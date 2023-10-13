use crate::http_api;

#[derive(Clone, Debug)]
pub struct Config {
    pub http_api: http_api::Config,
}
