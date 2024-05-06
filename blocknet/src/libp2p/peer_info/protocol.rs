// Copyright 2018 Parity Technologies (UK) Ltd.
//
// Permission is hereby granted, free of charge, to any person obtaining a
// copy of this software and associated documentation files (the "Software"),
// to deal in the Software without restriction, including without limitation
// the rights to use, copy, modify, merge, publish, distribute, sublicense,
// and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS
// OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
// FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

use libp2p::core::{multiaddr, Multiaddr};
use libp2p::identity;
use libp2p::identity::PublicKey;
use libp2p::swarm::StreamProtocol;
use std::convert::TryFrom;
use std::io;
use thiserror::Error;

const MAX_MESSAGE_SIZE_BYTES: usize = 4096;

pub const PROTOCOL_NAME: StreamProtocol = StreamProtocol::new("/ipfs/id/1.0.0");

pub const PUSH_PROTOCOL_NAME: StreamProtocol = StreamProtocol::new("/ipfs/id/push/1.0.0");

/// Identify information of a peer sent in protocol messages.
#[derive(Debug, Clone)]
pub struct Info {
    /// The public key of the local peer.
    pub public_key: PublicKey,
    /// Application-specific version of the protocol family used by the peer,
    /// e.g. `ipfs/1.0.0` or `polkadot/1.0.0`.
    pub protocol_version: String,
    /// Name and version of the peer, similar to the `User-Agent` header in
    /// the HTTP protocol.
    pub agent_version: String,
    /// The addresses that the peer is listening on.
    pub listen_addrs: Vec<Multiaddr>,
    /// The list of protocols supported by the peer, e.g. `/ipfs/ping/1.0.0`.
    pub protocols: Vec<StreamProtocol>,
    /// Address observed by or for the remote.
    pub observed_addr: Multiaddr,
}

impl Info {
    pub fn merge(&mut self, info: PushInfo) {
        if let Some(public_key) = info.public_key {
            self.public_key = public_key;
        }
        if let Some(protocol_version) = info.protocol_version {
            self.protocol_version = protocol_version;
        }
        if let Some(agent_version) = info.agent_version {
            self.agent_version = agent_version;
        }
        if let Some(listen_addrs) = info.listen_addrs {
            self.listen_addrs = listen_addrs;
        }
        if let Some(protocols) = info.protocols {
            self.protocols = protocols;
        }
        if let Some(observed_addr) = info.observed_addr {
            self.observed_addr = observed_addr;
        }
    }
}

/// Identify push information of a peer sent in protocol messages.
/// Note that missing fields should be ignored, as peers may choose to send partial updates containing only the fields whose values have changed.
#[derive(Debug, Clone)]
pub struct PushInfo {
    pub public_key: Option<PublicKey>,
    pub protocol_version: Option<String>,
    pub agent_version: Option<String>,
    pub listen_addrs: Option<Vec<Multiaddr>>,
    pub protocols: Option<Vec<StreamProtocol>>,
    pub observed_addr: Option<Multiaddr>,
}

impl From<Info> for PushInfo {
    fn from(info: Info) -> PushInfo {
        PushInfo {
            public_key: Some(info.public_key),
            protocol_version: Some(info.protocol_version),
            agent_version: Some(info.agent_version),
            listen_addrs: Some(info.listen_addrs),
            protocols: Some(info.protocols),
            observed_addr: Some(info.observed_addr),
        }
    }
}

fn parse_listen_addrs(listen_addrs: Vec<Vec<u8>>) -> Vec<Multiaddr> {
    listen_addrs
        .into_iter()
        .filter_map(|bytes| match Multiaddr::try_from(bytes) {
            Ok(a) => Some(a),
            Err(e) => {
                tracing::debug!("Unable to parse multiaddr: {e:?}");
                None
            }
        })
        .collect()
}

fn parse_protocols(protocols: Vec<String>) -> Vec<StreamProtocol> {
    protocols
        .into_iter()
        .filter_map(|p| match StreamProtocol::try_from_owned(p) {
            Ok(p) => Some(p),
            Err(e) => {
                tracing::debug!("Received invalid protocol from peer: {e}");
                None
            }
        })
        .collect()
}

fn parse_public_key(public_key: Option<Vec<u8>>) -> Option<PublicKey> {
    public_key.and_then(|key| match PublicKey::try_decode_protobuf(&key) {
        Ok(k) => Some(k),
        Err(e) => {
            tracing::debug!("Unable to decode public key: {e:?}");
            None
        }
    })
}

fn parse_observed_addr(observed_addr: Option<Vec<u8>>) -> Option<Multiaddr> {
    observed_addr.and_then(|bytes| match Multiaddr::try_from(bytes) {
        Ok(a) => Some(a),
        Err(e) => {
            tracing::debug!("Unable to parse observed multiaddr: {e:?}");
            None
        }
    })
}

#[derive(Debug, Error)]
pub enum UpgradeError {
    #[error("Codec error")]
    Codec(String),
    #[error("I/O interaction failed")]
    Io(#[from] io::Error),
    #[error("Stream closed")]
    StreamClosed,
    #[error("Failed decoding multiaddr")]
    Multiaddr(#[from] multiaddr::Error),
    #[error("Failed decoding public key")]
    PublicKey(#[from] identity::DecodingError),
}
