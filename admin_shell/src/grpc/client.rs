use proto_definitions::generated::core_bridge_service_client::CoreBridgeServiceClient;
use proto_definitions::generated::{CommandRequest, CommandResponse, StatusUpdate};
use tokio_stream::StreamExt;
use tonic::Request;

pub async fn test_grpc_connection() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = CoreBridgeServiceClient::connect("http://127.0.0.1:50052").await?;

    // === Phase 1: Receive 10 heartbeat updates ===
    println!("[Phase 1] Receiving initial 10 heartbeats...");
    let mut stream = client
        .stream_status_updates(Request::new(()))
        .await?
        .into_inner();

    for i in 1..=10 {
        if let Some(update) = stream.next().await {
            let update: StatusUpdate = update?;
            println!("Heartbeat {i}: {:?}", update);
        }
    }

    // === Phase 2: Restart the system ===
    println!("[Phase 2] Sending RESTART...");
    let restart_response = client
        .execute_command(Request::new(CommandRequest {
            command: "RESTART".into(),
        }))
        .await?;

    println!("Restart response: {:?}", restart_response.into_inner());

    // === Phase 3: Receive 5 more heartbeats ===
    println!("[Phase 3] Receiving 5 heartbeats after restart...");
    let mut stream = client
        .stream_status_updates(Request::new(()))
        .await?
        .into_inner();

    for i in 1..=5 {
        if let Some(update) = stream.next().await {
            let update: StatusUpdate = update?;
            println!("Post-Restart Heartbeat {i}: {:?}", update);
        }
    }

    // === Phase 4: Shutdown the system ===
    println!("[Phase 4] Sending SHUTDOWN...");
    let shutdown_response = client
        .execute_command(Request::new(CommandRequest {
            command: "SHUTDOWN".into(),
        }))
        .await?;

    println!("Shutdown response: {:?}", shutdown_response.into_inner());

    Ok(())
}
