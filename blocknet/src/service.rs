use futures::stream::Stream;
use std::future::Future;

pub trait NetworkService: Send {
    type PeerId;
    type Error;

    fn connected_peers(&self) -> impl IntoIterator<Item = Self::PeerId>;
}

pub struct NetworkEvent<PeerId, Message> {
    pub originator: PeerId,
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
