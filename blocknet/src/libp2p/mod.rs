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
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, RwLock},
};

pub type PeerId = libp2p::PeerId;
pub type ProtocolName = Cow<'static, str>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnyRequest {
    pub protocol_id: String,
    pub serialized: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnyResponse {
    pub protocol_id: String,
    pub serialized: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AnyMessage {
    pub protocol_name: ProtocolName,
    pub serialized: Vec<u8>,
}

#[derive(NetworkBehaviour)]
struct Behaviour<Info>
where
    Info: peer_info::Info + Serialize + DeserializeOwned + Send + 'static,
    Info::Push: Serialize + DeserializeOwned + Send + 'static,
{
    gossipsub: gossipsub::Behaviour,
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    identify: identify::Behaviour,
    peer_info: peer_info::json::Behaviour<Info>,
    mdns: mdns::tokio::Behaviour,
    request_response: request_response::json::Behaviour<AnyRequest, AnyResponse>,
}

enum ActionItem {
    Broadcast {
        message: AnyMessage,
    },
    Notify {
        peer_id: PeerId,
        message: AnyMessage,
    },
    Request {
        peer_id: PeerId,
        request: AnyRequest,
        receiver: mpsc::Receiver<AnyResponse>,
    },
}

pub enum Error {}

pub struct NetworkWorker<Info>
where
    Info: peer_info::Info + Serialize + DeserializeOwned + Send + 'static,
    Info::Push: Serialize + DeserializeOwned + Send + 'static,
{
    swarm: Swarm<Behaviour<Info>>,
    queue: mpsc::Receiver<ActionItem>,
}

pub struct PeerInfo<Extra> {
    extra: Extra,
}

pub struct NetworkService<PeerExtraInfo> {
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo<PeerExtraInfo>>>>,
    sender: mpsc::Sender<ActionItem>,
}
