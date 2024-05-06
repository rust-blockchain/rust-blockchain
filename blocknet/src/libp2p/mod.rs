pub mod peer_info;

use crate::{
    NetworkMessageService as NetworkMessageServiceT,
    NetworkRequestService as NetworkRequestServiceT, NetworkService as NetworkServiceT,
};
use futures::channel::mpsc;
use libp2p::{
    gossipsub, identify, kad, mdns, request_response,
    swarm::{NetworkBehaviour, Swarm},
};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub type PeerId = libp2p::PeerId;
pub type ProtocolName = Cow<'static, str>;

#[derive(NetworkBehaviour)]
struct Behaviour<Info, Req, Res>
where
    Info: peer_info::Info + Serialize + DeserializeOwned + Send + 'static,
    Info::Push: Serialize + DeserializeOwned + Send + 'static,
    Req: Serialize + DeserializeOwned + Send + 'static,
    Res: Serialize + DeserializeOwned + Send + 'static,
{
    gossipsub: gossipsub::Behaviour,
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    identify: identify::Behaviour,
    peer_info: peer_info::json::Behaviour<Info>,
    mdns: mdns::tokio::Behaviour,
    request_response: request_response::json::Behaviour<Req, Res>,
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

pub struct NetworkWorker<Info, Msg, Req, Res>
where
    Info: peer_info::Info + Serialize + DeserializeOwned + Send + 'static,
    Info::Push: Serialize + DeserializeOwned + Send + 'static,
    Req: Serialize + DeserializeOwned + Send + 'static,
    Res: Serialize + DeserializeOwned + Send + 'static,
{
    swarm: Swarm<Behaviour<Info, Req, Res>>,
    queue: mpsc::Receiver<ActionItem<Msg, Req, Res>>,
}

pub struct PeerInfo<Extra> {
    extra: Extra,
}

pub struct NetworkService<Msg, Req, Res, PeerExtraInfo> {
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo<PeerExtraInfo>>>>,
    sender: mpsc::Sender<ActionItem<Msg, Req, Res>>,
}
