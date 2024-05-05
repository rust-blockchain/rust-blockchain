use super::{Info, PushInfo, UpgradeError};
use async_trait::async_trait;
use futures::prelude::*;

/// A `Codec` defines the request and response types
/// for a request-response [`Behaviour`](crate::Behaviour) protocol or
/// protocol family and how they are encoded / decoded on an I/O stream.
#[async_trait]
pub trait Codec: Send + 'static {
    async fn read_info<T>(io: T) -> Result<Info, UpgradeError>
    where
        T: AsyncRead + Unpin + Send;

    async fn read_push_info<T>(io: T) -> Result<PushInfo, UpgradeError>
    where
        T: AsyncRead + Unpin + Send;

    async fn write_info<T>(io: T, info: Info) -> Result<(), UpgradeError>
    where
        T: AsyncWrite + Unpin + Send;

    async fn write_push_info<T>(io: T, info: PushInfo) -> Result<(), UpgradeError>
    where
        T: AsyncWrite + Unpin + Send;
}
