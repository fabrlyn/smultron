use crate::http;

#[derive(Clone, Debug)]
pub struct Config {
    pub http_api: http::Config,
}
