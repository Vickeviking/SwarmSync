use admin_shell::generated::core_bridge_service_client::CoreBridgeServiceClient;
use admin_shell::generated::{CommandRequest, StatusUpdate};
use tokio_stream::StreamExt;
use tonic::{transport::Channel, Request};

async fn connect_client() -> Result<CoreBridgeServiceClient<Channel>, Box<dyn std::error::Error>> {
    let core_host = std::env::var("CORE_HOST").unwrap_or("127.0.0.1".into());
    let core_port = std::env::var("CORE_PORT").unwrap_or("50052".into());
    let address = format!("http://{}:{}", core_host, core_port);
    Ok(CoreBridgeServiceClient::connect(address).await?)
}

#[tokio::test]
async fn test_heartbeat_restart_heartbeat() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = connect_client().await?;
    let mut stream = client
        .stream_status_updates(Request::new(()))
        .await?
        .into_inner();

    for i in 1..=3 {
        let hb = stream.next().await.ok_or("Missing heartbeat")??;
        println!("Heartbeat {i} before restart: {:?}", hb);
    }

    let _ = client
        .execute_command(Request::new(CommandRequest {
            command: "RESTART".into(),
        }))
        .await?;

    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    for i in 1..=3 {
        let hb = stream
            .next()
            .await
            .ok_or("Missing heartbeat after restart")??;
        println!("Heartbeat {i} after restart: {:?}", hb);
    }

    Ok(())
}
