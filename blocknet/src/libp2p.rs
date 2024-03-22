use libp2p::{
    gossipsub, identify, kad, mdns,
    swarm::{NetworkBehaviour, Swarm},
};
use std::borrow::Cow;

pub type ProtocolName = Cow<'static, str>;

#[derive(NetworkBehaviour)]
struct Behaviour {
    gossipsub: gossipsub::Behaviour,
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    identify: identify::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

pub struct NetworkWorker {
    swarm: Swarm<Behaviour>,
}
