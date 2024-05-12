use futures::stream::Stream;
use std::{future::Future, ops::Deref};

pub trait Service: Send {
    type PeerId;
    type PeerInfo;
    type Error;

    fn local_info(&self) -> impl Deref<Target = Self::PeerInfo>;
    fn set_local_info(&self, info: Self::PeerInfo);
    fn peers(&self) -> impl IntoIterator<Item = (Self::PeerId, Self::PeerInfo)>;
}

pub trait Message {
    type Topic;

    fn topic(&self) -> Self::Topic;
}

pub trait Event {
    type Origin;
    type Message: Message;

    fn origin(&self) -> impl Deref<Target = Self::Origin>;
    fn message(&self) -> impl Deref<Target = Self::Message>;
    fn into_message(self) -> Self::Message;
}

pub trait MessageService<Msg: Message>: Service {
    type Event: Event<Origin = Self::PeerId, Message = Msg>;

    fn listen(
        &self,
        topic: Msg::Topic,
    ) -> impl Stream<Item = Result<Self::Event, Self::Error>> + Send;
}

pub trait BroadcastService<Msg: Message>: MessageService<Msg> {
    fn broadcast(&self, message: Msg) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait NotifyService<Msg: Message>: MessageService<Msg> {
    fn notify(
        &self,
        peer: Self::PeerId,
        message: Msg,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait Request {
    type Response;
}

pub trait RequestService<Req: Request>: Service {
    fn request(
        &self,
        peer: Self::PeerId,
        request: Req,
    ) -> impl Future<Output = Result<Req::Response, Self::Error>> + Send;
}
