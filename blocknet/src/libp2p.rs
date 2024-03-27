use crate::{
    NetworkMessageService as NetworkMessageServiceT,
    NetworkRequestService as NetworkRequestServiceT, NetworkService as NetworkServiceT,
};
use futures::channel::mpsc;
use libp2p::{
    gossipsub, identify, kad, mdns,
    swarm::{NetworkBehaviour, Swarm},
};
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub type PeerId = libp2p::PeerId;
pub type ProtocolName = Cow<'static, str>;

#[derive(NetworkBehaviour)]
struct Behaviour {
    gossipsub: gossipsub::Behaviour,
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    identify: identify::Behaviour,
    mdns: mdns::tokio::Behaviour,
}

enum ActionItem<Msg, Req, Res> {
    Broadcast {
        protocol_name: ProtocolName,
        message: Msg,
    },
    Notify {
        protocol_name: ProtocolName,
        peer_id: PeerId,
        message: Msg,
    },
    Request {
        peer_id: PeerId,
        request: Req,
        receiver: mpsc::Receiver<Res>,
    },
}

pub enum Error {}

pub struct NetworkWorker<Msg, Req, Res> {
    swarm: Swarm<Behaviour>,
    queue: mpsc::Receiver<ActionItem<Msg, Req, Res>>,
}

pub struct PeerInfo<Extra> {
    extra: Extra,
}

pub struct NetworkService<Msg, Req, Res, PeerExtraInfo> {
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo<PeerExtraInfo>>>>,
    sender: mpsc::Sender<ActionItem<Msg, Req, Res>>,
}
