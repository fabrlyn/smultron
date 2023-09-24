use crate::device::Device;
use crate::AppResult;

use coapium::asynchronous::Client;
use coapium::codec::url::Endpoint;
use futures_util::{pin_mut, StreamExt};
use mdns::{discover, RecordKind};
use std::net::IpAddr;
use std::time::Duration;
use tokio::spawn;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::{info, instrument};

const ONE_MINUTE: Duration = Duration::from_secs(60);
const COAP_SERVICE: &'static str = "_coap._udp.local";

pub struct Gateway {
    devices: Vec<Device>,
}

impl Gateway {
    pub fn new() -> Self {
        Self { devices: vec![] }
    }

    pub async fn run(self) -> AppResult {
        self.discover().await
    }

    async fn discover(&self) -> AppResult {
        info!("starting service discovery for {:?}", COAP_SERVICE);

        let discovery_stream = discover::all(COAP_SERVICE, ONE_MINUTE)?.listen();
        pin_mut!(discovery_stream);

        while let Some(Ok(service)) = discovery_stream.next().await {
            self.on_discovered(service).await;
        }

        Ok(())
    }

    async fn on_discovered(&self, service: mdns::Response) {
       info!("service discovered: {:?}", service);

        let Some((ip, port)) = find_ip_and_port(&service) else {
            return;
        };

        let endpoint = format!("coap://{}:{}", ip, port)
            .as_str()
            .try_into()
            .unwrap();
        if self.device_by_endpoint_exist(&endpoint) {
            info!("device already registered for endpoint {}", endpoint);
            return;
        }

        self.register_device(endpoint).await;
    }

    async fn register_device(&self, endpoint: Endpoint) {
        let device = Device::new(endpoint).await;
    }

    fn device_by_endpoint_exist(&self, endpoint: &Endpoint) -> bool {
        self.devices
            .iter()
            .find(|d| d.endpoint() == *endpoint)
            .is_some()
    }
}

fn find_ip_and_port(service: &mdns::Response) -> Option<(IpAddr, u16)> {
    let Some(ip) = find_ip(service) else {
        return None;
    };
    let Some(port) = find_port(service) else {
        return None;
    };

    Some((ip, port))
}

fn find_ip(service: &mdns::Response) -> Option<IpAddr> {
    service.additional.iter().find_map(|a| match a.kind {
        RecordKind::A(ip) => Some(IpAddr::V4(ip)),
        RecordKind::AAAA(ip) => Some(IpAddr::V6(ip)),
        _ => None,
    })
}

fn find_port(service: &mdns::Response) -> Option<u16> {
    service.additional.iter().find_map(|a| match a.kind {
        RecordKind::SRV { port, .. } => Some(port),
        _ => None,
    })
}
