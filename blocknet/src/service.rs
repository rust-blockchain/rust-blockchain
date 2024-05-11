use futures::stream::Stream;
use std::future::Future;

pub trait Service: Send {
    type PeerId;
    type PeerInfo;
    type Error;

    fn local_info(&self) -> Self::PeerInfo;
    fn set_local_info(&self, info: Self::PeerInfo) -> Result<(), Self::Error>;
    fn peers(&self) -> impl IntoIterator<Item = (Self::PeerId, Self::PeerInfo)>;
}

pub trait EventWithOrigin {
    type Origin;

    fn origin(&self) -> &Self::Origin;
}

pub trait EventWithMessage {
    type Message;

    fn message(&self) -> &Self::Message;
    fn into_message(self) -> Self::Message;
}

pub trait EventWithReceiver {
    type Receiver;

    fn receiver(&self) -> &Self::Receiver;
}

pub trait MessageService<Msg>: Service {
    type Event: EventWithOrigin<Origin = Self::PeerId> + EventWithMessage<Message = Msg>;

    fn listen(&self) -> impl Stream<Item = Result<Self::Event, Self::Error>> + Send;
}

pub trait BroadcastService<Msg>: MessageService<Msg> {
    fn broadcast(&self, message: Msg) -> impl Future<Output = Result<(), Self::Error>> + Send;
}

pub trait NotifyService<Msg>: MessageService<Msg>
where
    Self::Event: EventWithReceiver<Receiver = Self::PeerId>,
{
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
