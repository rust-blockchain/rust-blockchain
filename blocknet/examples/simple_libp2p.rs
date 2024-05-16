use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PeerInfo {
    best_block: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let worker = blocknet::libp2p::Worker::new(PeerInfo { best_block: 1 })?;

    let _ = tokio::spawn(worker.run()).await?;

    Ok(())
}
