use futures::stream::Stream;
use std::{future::Future, ops::Deref};

pub trait Service: Send {
    type PeerId;
    type PeerInfo;
    type Error;

    fn local_info(&self) -> Self::PeerInfo;
    fn set_local_info(&mut self, info: Self::PeerInfo);
    fn peers(&self) -> impl IntoIterator<Item = (Self::PeerId, Self::PeerInfo)>;
}

pub trait Event {
    type Origin;
    type Value;

    fn origin(&self) -> impl Deref<Target = Self::Origin>;
    fn value(&self) -> impl Deref<Target = Self::Value>;
    fn into_value(self) -> Self::Value;
}

pub trait Message {
    type Topic;

    fn topic(&self) -> Self::Topic;
}

pub trait BroadcastService<Msg: Message>: Service {
    type Event: Event<Origin = Self::PeerId, Value = Msg>;

    fn listen(
        &mut self,
        topic: Msg::Topic,
    ) -> impl Stream<Item = Result<Self::Event, Self::Error>> + Send;
    fn broadcast(&mut self, message: Msg) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait NotifyService<Not>: Service {
    type Event: Event<Origin = Self::PeerId, Value = Not>;

    fn listen(&mut self) -> impl Stream<Item = Result<Self::Event, Self::Error>> + Send;
    fn notify(
        &mut self,
        peer: Self::PeerId,
        notification: Not,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait Request {
    type Response;
}

pub trait RequestService<Req: Request>: Service {
    type Event: Event<Origin = Self::PeerId, Value = Req>;
    type Channel;

    fn listen(
        &mut self,
    ) -> impl Future<Output = Result<(Self::Channel, Self::Event), Self::Error>> + Send;
    fn request(
        &mut self,
        peer: Self::PeerId,
        request: Req,
    ) -> impl Future<Output = Result<Req::Response, Self::Error>> + Send;
    fn respond(
        &mut self,
        channel: Self::Channel,
        response: Req::Response,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;
}
