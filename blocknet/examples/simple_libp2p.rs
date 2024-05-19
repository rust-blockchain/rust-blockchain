use blocknet::{BroadcastService, Message};
use futures::{select, stream::StreamExt, FutureExt};
use serde::{Deserialize, Serialize};
use tokio::{io, io::AsyncBufReadExt};
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PeerInfo {
    best_block: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SimpleBroadcast {
    message: String,
}

struct SimpleBroadcastTopic;

impl Message for SimpleBroadcast {
    type Topic = SimpleBroadcastTopic;

    fn topic(&self) -> Self::Topic {
        SimpleBroadcastTopic
    }
}

impl Into<String> for SimpleBroadcastTopic {
    fn into(self) -> String {
        "simple_broadcast_topic".into()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let worker = blocknet::libp2p::Worker::new(PeerInfo { best_block: 1 })?;
    let service = worker.service();
    let mut worker_handle = tokio::spawn(worker.run()).fuse();

    let mut stdin = io::BufReader::new(io::stdin()).lines();
    let mut stream_service = service.clone();
    let mut broadcast_stream = Box::pin(
        BroadcastService::<SimpleBroadcast>::listen(&mut stream_service, SimpleBroadcastTopic)
            .fuse(),
    );

    loop {
        let mut service = service.clone();

        select! {
            stdin_next = stdin.next_line().fuse() => {
                if let Ok(Some(line)) = stdin_next {
                    if line == "exit" {
                        break
                    }
                    match BroadcastService::<SimpleBroadcast>::broadcast(
                        &mut service,
                        SimpleBroadcast {
                            message: line,
                        },
                    ).await {
                        Ok(()) => info!("Broadcasted"),
                        Err(e) => info!("Broadcast error: {:?}", e),
                    }
                }
            },
            broadcast_next = broadcast_stream.select_next_some() => {
                match broadcast_next {
                    Ok(message) => info!("Received message: {:?}", message),
                    Err(e) => info!("Receive error: {:?}", e),
                }
            },
            worker_err = worker_handle => {
                if let Err(e) = worker_err {
                    error!("Worker returned: {:?}", e);
                    break
                }
            }
        }
    }

    Ok(())
}
