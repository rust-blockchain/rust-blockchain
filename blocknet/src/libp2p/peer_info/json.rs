use super::{Info, UpgradeError};
use async_trait::async_trait;
use futures::prelude::*;
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;

pub type Behaviour<TInfo> = super::Behaviour<TInfo, Codec<TInfo>>;

/// Max size in bytes
const SIZE_MAXIMUM: u64 = 1024 * 1024;

pub struct Codec<TInfo> {
    _marker: PhantomData<TInfo>,
}

#[async_trait]
impl<TInfo> super::Codec<TInfo> for Codec<TInfo>
where
    TInfo: Serialize + DeserializeOwned + Info,
    TInfo::Push: Serialize + DeserializeOwned,
{
    async fn read_info<T>(io: T) -> Result<TInfo, UpgradeError>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut vec = Vec::new();
        io.take(SIZE_MAXIMUM).read_to_end(&mut vec).await?;

        let info: TInfo = serde_json::from_slice(vec.as_slice())
            .map_err(|e| UpgradeError::Codec(format!("{:?}", e)))?;
        Ok(info)
    }

    async fn read_push_info<T>(io: T) -> Result<TInfo::Push, UpgradeError>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut vec = Vec::new();
        io.take(SIZE_MAXIMUM).read_to_end(&mut vec).await?;

        let info: TInfo::Push = serde_json::from_slice(vec.as_slice())
            .map_err(|e| UpgradeError::Codec(format!("{:?}", e)))?;
        Ok(info)
    }

    async fn write_info<T>(mut io: T, info: TInfo) -> Result<(), UpgradeError>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let info: TInfo = info.into();
        let data =
            serde_json::to_vec(&info).map_err(|e| UpgradeError::Codec(format!("{:?}", e)))?;

        io.write_all(data.as_ref()).await?;
        Ok(())
    }

    async fn write_push_info<T>(mut io: T, info: TInfo::Push) -> Result<(), UpgradeError>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let info: TInfo::Push = info.into();
        let data =
            serde_json::to_vec(&info).map_err(|e| UpgradeError::Codec(format!("{:?}", e)))?;

        io.write_all(data.as_ref()).await?;
        Ok(())
    }
}
