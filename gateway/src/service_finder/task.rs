use std::{
    future::ready,
    time::{Duration, Instant},
};

use futures_util::{pin_mut, StreamExt};
use mdns::{discover, Response};
use tokio::{pin, select, sync::oneshot};

use crate::{
    service::{Name, Service},
    service_finder::worker::Msg,
};

use super::worker::Actor;

pub type StopRx = oneshot::Receiver<()>;
pub type StopTx = oneshot::Sender<()>;

fn find_services(response: Response) -> Vec<Service> {
    response.records().filter_map(into_service).collect()
}

fn into_service(record: &mdns::Record) -> Option<Service> {
    match &record.kind {
        mdns::RecordKind::SRV { port, target, .. } => Some(Service {
            found_at: Instant::now(),
            name: record.name.clone(),
            port: *port,
            target: target.clone(),
        }),
        _ => None,
    }
}

fn on_next(actor: &Actor, response: Option<Response>) -> bool {
    let Some(response) = response else {
        return true;
    };

    for service in find_services(response) {
        actor.send_message(Msg::Found(service));
    }

    false
}

pub async fn start(actor: Actor, stop_rx: StopRx, name: Name) {
    let Ok(discovery) = discover::all(name, Duration::from_secs(15)) else {
        return;
    };

    let stream = discovery.listen().filter_map(|result| ready(result.ok()));

    pin!(stop_rx);
    pin_mut!(stream);

    loop {
        let should_stop = select! {
            result = stream.next() => on_next(&actor, result),
            _ = &mut stop_rx => true
        };

        if should_stop {
            actor.send_message(Msg::TaskStopping);
            return;
        }
    }
}
