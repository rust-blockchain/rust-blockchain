use futures::stream::Stream;
use std::future::Future;

pub trait NetworkService: Send {
    type PeerId;
    type PeerInfo;
    type Error;

    fn local_info(&self) -> Self::PeerInfo;
    fn set_local_info(&self, info: Self::PeerInfo) -> Result<(), Self::Error>;
    fn peers(&self) -> impl IntoIterator<Item = (Self::PeerId, Self::PeerInfo)>;
}

pub struct NetworkEvent<PeerId, Message> {
    pub origin: PeerId,
    pub message: Message,
}

pub trait NetworkMessageService<Message>: NetworkService {
    type ProtocolName;

    fn broadcast(
        &self,
        name: Self::ProtocolName,
        message: Message,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn notify(
        &self,
        peer: Self::PeerId,
        message: Message,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send;

    fn listen(
        &self,
        name: Self::ProtocolName,
    ) -> impl Stream<Item = Result<NetworkEvent<Self::PeerId, Message>, Self::Error>> + Send;
}

pub trait NetworkRequestService<Request, Response>: NetworkService {
    fn request(
        &self,
        peer: Self::PeerId,
        request: Request,
    ) -> impl Future<Output = Result<Response, Self::Error>> + Send;
}
