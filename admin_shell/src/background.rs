use crate::generated::core_bridge_service_client::CoreBridgeServiceClient;
use crate::generated::{CommandRequest, StatusUpdate};

use tokio::sync::mpsc::Sender;
use tokio_stream::StreamExt;
use tonic::{transport::Channel, Request};

pub async fn connect_client() -> Result<CoreBridgeServiceClient<Channel>, Box<dyn std::error::Error>>
{
    let core_host = std::env::var("CORE_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let core_port = std::env::var("CORE_PORT").unwrap_or_else(|_| "50052".into());
    let address = format!("http://{}:{}", core_host, core_port);
    Ok(CoreBridgeServiceClient::connect(address).await?)
}

pub async fn start_heartbeat_listener(
    tx: Sender<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut client = connect_client().await?;
    let response = client.stream_status_updates(Request::new(())).await?;
    let mut stream = response.into_inner();

    while let Some(msg) = stream.next().await {
        let status: StatusUpdate = msg?;
        tx.send(format!("💓 {:?}", status)).await?;
    }

    Ok(())
}
