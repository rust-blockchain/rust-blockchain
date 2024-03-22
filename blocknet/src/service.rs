use futures::stream::Stream;
use std::pin::Pin;

pub trait NetworkService {
    type PeerId;
    type ProtocolName;
    type Error;
}

pub struct NetworkEvent<PeerId, Message> {
    pub originator: PeerId,
    pub message: Message,
}

pub trait NetworkMessageService: NetworkService {
    type Message;

    async fn broadcast(
        &self,
        name: Self::ProtocolName,
        message: Self::Message,
    ) -> Result<(), Self::Error>;

    async fn notify(&self, peer: Self::PeerId, message: Self::Message) -> Result<(), Self::Error>;

    fn listen(
        &self,
        name: Self::ProtocolName,
    ) -> Pin<
        Box<
            dyn Stream<Item = Result<NetworkEvent<Self::PeerId, Self::Message>, Self::Error>>
                + Send,
        >,
    >;
}

pub trait NetworkRequestService: NetworkService {
    type Request;
    type Response;

    async fn request(
        &self,
        peer: Self::PeerId,
        request: Self::Request,
    ) -> Result<Self::Response, Self::Error>;
}
