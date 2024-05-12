pub mod peer_info;

use crate::{
    BroadcastService as BroadcastServiceT, Event as EventT, MessageService as MessageServiceT,
    Service as ServiceT,
};
use futures::{
    channel::mpsc,
    sink::SinkExt,
    stream::{Stream, TryStreamExt},
};
use libp2p::{
    gossipsub, identify, kad, mdns, request_response,
    swarm::{NetworkBehaviour, Swarm},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    convert::Infallible,
    future::Future,
    ops::Deref,
    sync::{Arc, Mutex, RwLock},
};
use sync_extra::{MutexExtra, RwLockExtra};
use thiserror::Error;

const MESSAGE_CHANNEL_BUFFER_SIZE: usize = 16;

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
    pub topic: &'static str,
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

#[derive(Debug, Error)]
pub enum Error {
    #[error("Codec error")]
    Codec(String),
    #[error("Sender channel error")]
    ChannelSend(#[from] mpsc::SendError),
}

pub struct NetworkWorker<Info>
where
    Info: peer_info::Info + Serialize + DeserializeOwned + Send + 'static,
    Info::Push: Serialize + DeserializeOwned + Send + 'static,
{
    swarm: Swarm<Behaviour<Info>>,
    queue: mpsc::Receiver<ActionItem>,
}

impl<Info> NetworkWorker<Info>
where
    Info: peer_info::Info + Serialize + DeserializeOwned + Send + 'static,
    Info::Push: Serialize + DeserializeOwned + Send + 'static,
{
    pub async fn run(mut self) -> Result<Infallible, Error> {
        loop {
            match self.step().await {
                Ok(()) => (),
                Err(err) => return Err(err),
            }
        }
    }

    pub async fn step(&mut self) -> Result<(), Error> {
        unimplemented!()
    }
}

#[derive(Clone, Debug)]
pub struct PeerInfo<Extra> {
    extra: Extra,
}

pub struct Service<PeerExtraInfo> {
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo<PeerExtraInfo>>>>,
    local_info: Arc<RwLock<PeerInfo<PeerExtraInfo>>>,
    listen_senders:
        Arc<Mutex<HashMap<&'static str, mpsc::Sender<Result<(PeerId, AnyMessage), Error>>>>>,
    action_sender: mpsc::Sender<ActionItem>,
}

impl<PeerExtraInfo> ServiceT for Service<PeerExtraInfo>
where
    PeerExtraInfo: Clone + Send + Sync + 'static,
{
    type PeerId = PeerId;
    type PeerInfo = PeerInfo<PeerExtraInfo>;
    type Error = Error;

    fn local_info(&self) -> impl Deref<Target = Self::PeerInfo> {
        self.local_info.read_unwrap()
    }

    fn set_local_info(&self, info: Self::PeerInfo) {
        *self.local_info.write_unwrap() = info;
    }

    fn peers(&self) -> impl IntoIterator<Item = (Self::PeerId, Self::PeerInfo)> {
        self.peers.read_unwrap().clone()
    }
}

pub struct Event<Msg> {
    origin: PeerId,
    message: Msg,
}

impl<Msg> EventT for Event<Msg> {
    type Origin = PeerId;
    type Message = Msg;

    fn origin(&self) -> impl Deref<Target = Self::Origin> {
        &self.origin
    }

    fn message(&self) -> impl Deref<Target = Msg> {
        &self.message
    }

    fn into_message(self) -> Msg {
        self.message
    }
}

pub trait Message: Send + Clone + Serialize + DeserializeOwned + 'static {
    const TOPIC: &'static str;
}

impl<PeerExtraInfo, Msg> MessageServiceT<Msg> for Service<PeerExtraInfo>
where
    PeerExtraInfo: Clone + Send + Sync + 'static,
    Msg: Message,
{
    type Event = Event<Msg>;

    fn listen(&self) -> impl Stream<Item = Result<Self::Event, Self::Error>> + Send {
        let (sender, receiver) = mpsc::channel(MESSAGE_CHANNEL_BUFFER_SIZE);

        self.listen_senders.lock_unwrap().insert(Msg::TOPIC, sender);

        let receiver: mpsc::Receiver<Result<(PeerId, AnyMessage), Error>> = receiver;
        receiver.and_then(|(origin, msg)| async move {
            Ok(Event {
                origin,
                message: serde_json::from_slice(&msg.serialized)
                    .map_err(|e| Error::Codec(format!("{:?}", e)))?,
            })
        })
    }
}

impl<PeerExtraInfo, Msg> BroadcastServiceT<Msg> for Service<PeerExtraInfo>
where
    PeerExtraInfo: Clone + Send + Sync + 'static,
    Msg: Message,
{
    fn broadcast(&self, message: Msg) -> impl Future<Output = Result<(), Self::Error>> + Send {
        let mut action_sender = self.action_sender.clone();

        async move {
            let item = ActionItem::Broadcast {
                message: AnyMessage {
                    topic: Msg::TOPIC,
                    serialized: serde_json::to_vec(&message)
                        .map_err(|e| Error::Codec(format!("{:?}", e)))?,
                },
            };

            action_sender.send(item).await?;
            Ok(())
        }
    }
}
