use super::{Info, UpgradeError};
use async_trait::async_trait;
use futures::prelude::*;

/// A `Codec` defines the request and response types
/// for a request-response [`Behaviour`](crate::Behaviour) protocol or
/// protocol family and how they are encoded / decoded on an I/O stream.
#[async_trait]
pub trait Codec<TInfo: Info>: Send + 'static {
    async fn read_info<T>(io: T) -> Result<TInfo, UpgradeError>
    where
        T: AsyncRead + Unpin + Send;

    async fn read_push_info<T>(io: T) -> Result<TInfo::Push, UpgradeError>
    where
        T: AsyncRead + Unpin + Send;

    async fn write_info<T>(io: T, info: TInfo) -> Result<(), UpgradeError>
    where
        T: AsyncWrite + Unpin + Send;

    async fn write_push_info<T>(io: T, info: TInfo::Push) -> Result<(), UpgradeError>
    where
        T: AsyncWrite + Unpin + Send;
}
