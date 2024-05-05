use async_trait::async_trait;
use futures::prelude::*;
use libp2p::swarm::StreamProtocol;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{io, marker::PhantomData};

#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Identify {
    pub protocol_version: Option<String>,
    pub agent_version: Option<String>,
    pub public_key: Option<String>,
    pub listen_addrs: Vec<String>,
    pub observed_addr: Option<String>,
    pub protocols: Vec<String>,
}

/// Max request size in bytes
const REQUEST_SIZE_MAXIMUM: u64 = 1024 * 1024;
/// Max response size in bytes
const RESPONSE_SIZE_MAXIMUM: u64 = 10 * 1024 * 1024;

pub struct Codec<Req, Resp> {
    phantom: PhantomData<(Req, Resp)>,
}

impl<Req, Resp> Default for Codec<Req, Resp> {
    fn default() -> Self {
        Codec {
            phantom: PhantomData,
        }
    }
}

impl<Req, Resp> Clone for Codec<Req, Resp> {
    fn clone(&self) -> Self {
        Self::default()
    }
}

#[async_trait]
impl<Req, Resp> crate::Codec for Codec<Req, Resp>
where
    Req: Send + Serialize + DeserializeOwned,
    Resp: Send + Serialize + DeserializeOwned,
{
    type Protocol = StreamProtocol;
    type Request = Req;
    type Response = Resp;

    async fn read_request<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Req>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut vec = Vec::new();

        io.take(REQUEST_SIZE_MAXIMUM).read_to_end(&mut vec).await?;

        Ok(serde_json::from_slice(vec.as_slice())?)
    }

    async fn read_response<T>(&mut self, _: &Self::Protocol, io: &mut T) -> io::Result<Resp>
    where
        T: AsyncRead + Unpin + Send,
    {
        let mut vec = Vec::new();

        io.take(RESPONSE_SIZE_MAXIMUM).read_to_end(&mut vec).await?;

        Ok(serde_json::from_slice(vec.as_slice())?)
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&req)?;

        io.write_all(data.as_ref()).await?;

        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        resp: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let data = serde_json::to_vec(&resp)?;

        io.write_all(data.as_ref()).await?;

        Ok(())
    }
}
