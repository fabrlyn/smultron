use std::error::Error;

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
use tracing::info;
use web_linking::links;

use crate::ipso::{self, ResourceInstance};

pub struct Device {
    endpoint: Endpoint,
    client: coapium::asynchronous::Client,
    resource_instances: Vec<ResourceInstance>,
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
        info!("response {:?}", payload);

        // Could be done better,
        let resource_instances = links(payload.as_str())
            .unwrap()
            .into_iter()
            .map(|l| l.value.value)
            .map(ipso::ResourceInstance::from_str)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();

        info!("resource instances: {:?}", resource_instances);

        Ok(Self {
            endpoint,
            client,
            resource_instances,
        })
    }

    pub async fn poll(&self) {
        for resource_instance in &self.resource_instances {
            let path = resource_instance.to_path();

            let mut options = GetOptions::new();
            options.set_uri_path(path.clone().try_into().unwrap());
            let response = self
                .client
                .execute(NewRequest::Get(Get {
                    options,
                    reliability: Reliability::Confirmable(ConfirmableParameters::default(
                        initial_retransmission_factor(),
                    )),
                }))
                .await;

            let payload = resource_instance.parse_payload(response.unwrap().payload);
            println!("{}{} => {:?}", self.endpoint, path, payload);
        }
    }
}

// TODO: Make this better in coapium crate
fn initial_retransmission_factor() -> InitialRetransmissionFactor {
    InitialRetransmissionFactor::new(thread_rng().gen_range(0.0..1.0)).unwrap()
}
