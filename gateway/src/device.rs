use std::error::Error;
use std::net::IpAddr;

use coapium::{
    asynchronous::Client,
    codec::{message::GetOptions, url::Endpoint},
    protocol::{
        get::Get,
        new_request::NewRequest,
        reliability::Reliability,
        transmission_parameters::{ConfirmableParameters, InitialRetransmissionFactor},
    },
};
use rand::{thread_rng, Rng};

pub struct Device {
    endpoint: Endpoint,
    client: coapium::asynchronous::Client,
}

impl Device {
    pub fn endpoint(&self) -> Endpoint {
        // TODO: Could be improved in coapium crate, add to Client
        self.endpoint.clone()
    }

    pub async fn new(endpoint: Endpoint) -> Result<Self, Box<dyn Error>> {
        let client = Client::new(endpoint.clone()).await;
        let mut options = GetOptions::new();
        options.set_uri_path("/.well-known/core".try_into().unwrap());
        let response = client
            .execute(NewRequest::Get(Get {
                options,
                reliability: Reliability::Confirmable(ConfirmableParameters::default(
                    initial_retransmission_factor(),
                )),
            }))
            .await
            .map_err(|_| "invalid well known request")?;

        let payload = String::from_utf8(response.payload.value().to_vec()).unwrap();
        println!("response {:?}", payload);
        // TODO: parse core link format

        Ok(Self { endpoint, client })
    }
}

// TODO: Make this better in coapium crate
fn initial_retransmission_factor() -> InitialRetransmissionFactor {
    InitialRetransmissionFactor::new(thread_rng().gen_range(0.0..1.0)).unwrap()
}
