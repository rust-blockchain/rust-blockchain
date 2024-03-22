use futures::stream::Stream;
use std::pin::Pin;

pub trait NetworkService {
    type PeerId;
    type Error;
}

pub struct NetworkEvent<PeerId, Message> {
    pub originator: PeerId,
    pub message: Message,
}

pub trait NetworkMessageService<Message>: NetworkService {
    type ProtocolName;

    async fn broadcast(
        &self,
        name: Self::ProtocolName,
        message: Message,
    ) -> Result<(), Self::Error>;

    async fn notify(&self, peer: Self::PeerId, message: Message) -> Result<(), Self::Error>;

    fn listen(
        &self,
        name: Self::ProtocolName,
    ) -> Pin<Box<dyn Stream<Item = Result<NetworkEvent<Self::PeerId, Message>, Self::Error>> + Send>>;
}

pub trait NetworkRequestService<Request, Response>: NetworkService {
    async fn request(&self, peer: Self::PeerId, request: Request) -> Result<Response, Self::Error>;
}
