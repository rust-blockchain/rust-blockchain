use super::{Info, PushInfo, UpgradeError};
use async_trait::async_trait;
use futures::prelude::*;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CodecInfo {
    pub protocol_version: String,
    pub agent_version: String,
    pub public_key: String,
    pub listen_addrs: Vec<String>,
    pub observed_addr: String,
    pub protocols: Vec<String>,
}

impl TryFrom<CodecInfo> for Info {
    type Error = UpgradeError;

    fn try_from(info: CodecInfo) -> Result<Info, UpgradeError> {
        todo!()
    }
}

impl From<Info> for CodecInfo {
    fn from(info: Info) -> CodecInfo {
        todo!()
    }
}

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct CodecPushInfo {
    pub protocol_version: Option<String>,
    pub agent_version: Option<String>,
    pub public_key: Option<String>,
    pub listen_addrs: Option<Vec<String>>,
    pub observed_addr: Option<String>,
    pub protocols: Option<Vec<String>>,
}

impl TryFrom<CodecPushInfo> for PushInfo {
    type Error = UpgradeError;

    fn try_from(info: CodecPushInfo) -> Result<PushInfo, UpgradeError> {
        todo!()
    }
}

impl From<PushInfo> for CodecPushInfo {
    fn from(info: PushInfo) -> CodecPushInfo {
        todo!()
    }
}

/// Max request size in bytes
const REQUEST_SIZE_MAXIMUM: u64 = 1024 * 1024;
/// Max response size in bytes
const RESPONSE_SIZE_MAXIMUM: u64 = 10 * 1024 * 1024;

pub struct Codec {
    _marker: PhantomData<()>,
}

#[async_trait]
impl super::Codec for Codec {
    async fn read_info<T>(io: T) -> Result<Info, UpgradeError>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut vec = Vec::new();
        io.take(REQUEST_SIZE_MAXIMUM).read_to_end(&mut vec).await?;

        let info: CodecInfo = serde_json::from_slice(vec.as_slice())
            .map_err(|e| UpgradeError::Codec(format!("{:?}", e)))?;
        Ok(info.try_into()?)
    }

    async fn read_push_info<T>(io: T) -> Result<PushInfo, UpgradeError>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut vec = Vec::new();
        io.take(REQUEST_SIZE_MAXIMUM).read_to_end(&mut vec).await?;

        let info: CodecPushInfo = serde_json::from_slice(vec.as_slice())
            .map_err(|e| UpgradeError::Codec(format!("{:?}", e)))?;
        Ok(info.try_into()?)
    }

    async fn write_info<T>(mut io: T, info: Info) -> Result<(), UpgradeError>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let info: CodecInfo = info.into();
        let data =
            serde_json::to_vec(&info).map_err(|e| UpgradeError::Codec(format!("{:?}", e)))?;

        io.write_all(data.as_ref()).await?;
        Ok(())
    }

    async fn write_push_info<T>(mut io: T, info: PushInfo) -> Result<(), UpgradeError>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let info: CodecPushInfo = info.into();
        let data =
            serde_json::to_vec(&info).map_err(|e| UpgradeError::Codec(format!("{:?}", e)))?;

        io.write_all(data.as_ref()).await?;
        Ok(())
    }
}
