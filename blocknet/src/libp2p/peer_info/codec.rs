use super::{Info, PushInfo};
use async_trait::async_trait;
use futures::prelude::*;
use std::io;

/// A `Codec` defines the request and response types
/// for a request-response [`Behaviour`](crate::Behaviour) protocol or
/// protocol family and how they are encoded / decoded on an I/O stream.
#[async_trait]
pub trait Codec: Send + 'static {
    async fn read_info<T>(&self, io: &mut T) -> io::Result<Info>
    where
        T: AsyncRead + Unpin + Send;

    async fn read_push_info<T>(&self, io: &mut T) -> io::Result<PushInfo>
    where
        T: AsyncRead + Unpin + Send;

    async fn write_info<T>(&self, io: &mut T, info: Info) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send;

    async fn write_push_info<T>(&self, io: &mut T, info: PushInfo) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send;
}
