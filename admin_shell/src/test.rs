mod grpc;

#[tokio::main]
async fn main() {
    if let Err(e) = grpc::client::test_grpc_connection().await {
        eprintln!("gRPC test failed: {}", e);
    }
}
