use crate::generated::core_bridge_service_client::CoreBridgeServiceClient;
use tonic::transport::Channel;

pub async fn connect_client() -> Result<CoreBridgeServiceClient<Channel>, Box<dyn std::error::Error>>
{
    let core_host = std::env::var("CORE_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let core_port = std::env::var("CORE_PORT").unwrap_or_else(|_| "50052".into());
    let address = format!("http://{}:{}", core_host, core_port);
    Ok(CoreBridgeServiceClient::connect(address).await?)
}
