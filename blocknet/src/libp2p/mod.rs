pub mod peer_info;

use crate::{
    BroadcastService as BroadcastServiceT, Event as EventT, Message as MessageT,
    Service as ServiceT,
};
use futures::{
    channel::mpsc,
    future::TryFutureExt,
    select,
    sink::SinkExt,
    stream::{Stream, StreamExt, TryStreamExt},
};
use libp2p::{
    gossipsub, identify, kad, mdns, request_response,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    borrow::Cow,
    collections::HashMap,
    convert::Infallible,
    fmt::Debug,
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
    pub topic: String,
    pub serialized: Vec<u8>,
}

enum ActionItem {
    BroadcastSend {
        message: AnyMessage,
    },
    BroadcastListen {
        sender: mpsc::Sender<Result<(PeerId, AnyMessage), Error>>,
        topic: String,
    },
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Codec error")]
    Codec(String),
    #[error("Sender channel error")]
    ChannelSend(#[from] mpsc::SendError),
    #[error("Gossipsub subscription")]
    GossipsubSubscription(#[from] gossipsub::SubscriptionError),
    #[error("Gossipsub publish")]
    GossipsubPublish(#[from] gossipsub::PublishError),

    #[error("Broadcast message with an unknown source")]
    UnknownOriginBroadcast(AnyMessage),
}

#[derive(NetworkBehaviour)]
struct Behaviour<PeerExtraInfo>
where
    PeerExtraInfo: Debug + Clone + Serialize + DeserializeOwned + Send + 'static,
{
    gossipsub: gossipsub::Behaviour,
    kademlia: kad::Behaviour<kad::store::MemoryStore>,
    identify: identify::Behaviour,
    peer_info: peer_info::json::Behaviour<PeerInfo<PeerExtraInfo>>,
    mdns: mdns::tokio::Behaviour,
    request_response: request_response::json::Behaviour<AnyRequest, AnyResponse>,
}

pub struct Worker<PeerExtraInfo>
where
    PeerExtraInfo: Debug + Clone + Serialize + DeserializeOwned + Send + 'static,
{
    swarm: Swarm<Behaviour<PeerExtraInfo>>,
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo<PeerExtraInfo>>>>,
    local_info: Arc<RwLock<PeerInfo<PeerExtraInfo>>>,
    broadcast_listen_senders: HashMap<
        gossipsub::TopicHash,
        (
            String,
            Vec<mpsc::Sender<Result<(PeerId, AnyMessage), Error>>>,
        ),
    >,
    action_receiver: mpsc::Receiver<ActionItem>,
}

impl<PeerExtraInfo> Worker<PeerExtraInfo>
where
    PeerExtraInfo: Debug + Clone + Serialize + DeserializeOwned + Send + 'static,
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
        select! {
            action = self.action_receiver.select_next_some() => {
                match action {
                    ActionItem::BroadcastSend {
                        message
                    } => {
                        let topic = gossipsub::IdentTopic::new(message.topic);
                        self.swarm.behaviour_mut().gossipsub
                            .publish(topic, message.serialized)?;
                    },
                    ActionItem::BroadcastListen {
                        sender, topic,
                    } => {
                        let ident_topic = gossipsub::IdentTopic::new(topic.clone());
                        self.swarm.behaviour_mut().gossipsub
                            .subscribe(&ident_topic)?;

                        self.broadcast_listen_senders.entry(ident_topic.hash())
                            .or_insert((topic, Vec::new()))
                            .1
                            .push(sender);
                    },
                }
            },
            event = self.swarm.select_next_some() => {
                match event {
                    SwarmEvent::Behaviour(BehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        message, ..
                    })) => {
                        if let Some(entry) = self.broadcast_listen_senders.get_mut(&message.topic) {
                            entry.1.retain(|sender| sender.is_closed());

                            let topic = entry.0.clone();
                            let any_message = AnyMessage {
                                topic,
                                serialized: message.data,
                            };

                            if let Some(source) = message.source {
                                for sender in &mut entry.1 {
                                    sender.send(Ok((source, any_message.clone()))).await?;
                                }
                            } else {
                                for sender in &mut entry.1 {
                                    sender.send(Err(Error::UnknownOriginBroadcast(any_message.clone()))).await?;
                                }
                            }
                        }
                    },
                    _ => (),
                }
            },
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PeerInfo<Extra> {
    extra: Extra,
}

impl<Extra> peer_info::Info for PeerInfo<Extra>
where
    Extra: Debug + Clone + Send + 'static,
{
    type Push = Self;

    fn merge(&mut self, push: Self::Push) {
        *self = push;
    }
}

pub struct Service<PeerExtraInfo> {
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo<PeerExtraInfo>>>>,
    local_info: Arc<RwLock<PeerInfo<PeerExtraInfo>>>,
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

    fn set_local_info(&mut self, info: Self::PeerInfo) {
        *self.local_info.write_unwrap() = info;
    }

    fn peers(&self) -> impl IntoIterator<Item = (Self::PeerId, Self::PeerInfo)> {
        self.peers.read_unwrap().clone()
    }
}

pub struct Event<Value> {
    origin: PeerId,
    value: Value,
}

impl<Value> EventT for Event<Value> {
    type Origin = PeerId;
    type Value = Value;

    fn origin(&self) -> impl Deref<Target = Self::Origin> {
        &self.origin
    }

    fn value(&self) -> impl Deref<Target = Value> {
        &self.value
    }

    fn into_value(self) -> Value {
        self.value
    }
}

impl<PeerExtraInfo, Msg> BroadcastServiceT<Msg> for Service<PeerExtraInfo>
where
    PeerExtraInfo: Clone + Send + Sync + 'static,
    Msg: MessageT + Send + Clone + Serialize + DeserializeOwned + 'static,
    Msg::Topic: Send + Into<String> + 'static,
{
    type Event = Event<Msg>;

    fn listen(
        &mut self,
        topic: Msg::Topic,
    ) -> impl Stream<Item = Result<Self::Event, Self::Error>> + Send {
        let (sender, receiver) = mpsc::channel(MESSAGE_CHANNEL_BUFFER_SIZE);

        async move {
            self.action_sender
                .send(ActionItem::BroadcastListen {
                    topic: topic.into(),
                    sender,
                })
                .await?;

            Ok(receiver.and_then(|(origin, msg)| async move {
                Ok(Event {
                    origin,
                    value: serde_json::from_slice(&msg.serialized)
                        .map_err(|e| Error::Codec(format!("{:?}", e)))?,
                })
            }))
        }
        .try_flatten_stream()
    }

    fn broadcast(&mut self, message: Msg) -> impl Future<Output = Result<(), Self::Error>> + Send {
        async move {
            let item = ActionItem::BroadcastSend {
                message: AnyMessage {
                    topic: message.topic().into(),
                    serialized: serde_json::to_vec(&message)
                        .map_err(|e| Error::Codec(format!("{:?}", e)))?,
                },
            };

            self.action_sender.send(item).await?;
            Ok(())
        }
    }
}
