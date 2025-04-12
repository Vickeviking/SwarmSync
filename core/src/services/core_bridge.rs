use crate::enums::system::{CoreEvent, Pulse};
use futures::stream::Stream;
use proto_definitions::generated::{
    core_bridge_service_server::CoreBridgeService,
    core_bridge_service_server::CoreBridgeServiceServer, CommandRequest, CommandResponse,
    StatusUpdate,
};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::signal;
use tokio::sync::{broadcast::Receiver, mpsc::UnboundedSender, Mutex};
use tokio_stream::{wrappers, wrappers::BroadcastStream, StreamExt};
use tonic::{transport::Server, Request, Response, Status};

pub type StatusUpdateStream = Pin<Box<dyn Stream<Item = Result<StatusUpdate, Status>> + Send>>;

// CoreBridge main struct
pub struct CoreBridge {
    core_event_manip_tx: UnboundedSender<CoreEvent>,
    pulse_rx: Option<Receiver<Pulse>>,
}

// CoreBridge constructor
impl CoreBridge {
    pub fn new(
        core_event_manip_tx: UnboundedSender<CoreEvent>,
        pulse_rx: Option<Receiver<Pulse>>,
    ) -> Self {
        Self {
            core_event_manip_tx,
            pulse_rx,
        }
    }

    pub async fn handle_command_logic(&self, command_string: String) -> CommandResponse {
        let commands: Vec<&str> = command_string.split(',').collect();

        for command in commands {
            match command.trim() {
                "STARTUP" => {
                    println!("Received STARTUP command");
                    let _ = self.core_event_manip_tx.send(CoreEvent::Startup);
                }
                "RESTART" => {
                    println!("Received RESTART command");
                    let _ = self.core_event_manip_tx.send(CoreEvent::Restart);
                }
                "SHUTDOWN" => {
                    println!("Received SHUTDOWN command");
                    let _ = self.core_event_manip_tx.send(CoreEvent::Shutdown);
                }
                _ => {
                    println!("Unknown command: {}", command);
                }
            }
        }

        CommandResponse {
            status: "success".into(),
            result: "Command executed successfully.".into(),
        }
    }

    pub fn generate_status_stream(&mut self) -> Result<StatusUpdateStream, Status> {
        let rx = self
            .pulse_rx
            .take()
            .ok_or_else(|| Status::internal("pulse_rx already consumed or not initialized"))?;

        let stream = BroadcastStream::new(rx).filter_map(|result| match result {
            Ok(_) | Err(wrappers::errors::BroadcastStreamRecvError::Lagged(_)) => {
                Some(Ok(StatusUpdate {
                    update_type: "heartbeat".to_string(),
                    value: 1,
                }))
            }
        });

        Ok(Box::pin(stream))
    }

    pub async fn init(core_bridge: Arc<Mutex<Self>>, mut core_event_rx: Receiver<CoreEvent>) {
        let listener = TcpListener::bind("0.0.0.0:5106")
            .await
            .expect("Failed to bind port");
        let active_users = Arc::new(Mutex::new(HashMap::new()));

        loop {
            tokio::select! {
                Ok((socket, addr)) = listener.accept() => {
                    println!("New connection from: {}", addr);
                    let users = Arc::clone(&active_users);
                    let core_event_tx = core_bridge.lock().await.core_event_manip_tx.clone();
                    tokio::spawn(Self::handle_client(socket, addr.to_string(), users, core_event_tx));
                },

                event = core_event_rx.recv() => {
                    match event {
                        Ok(CoreEvent::Startup) => println!("CoreBridge Startup event received."),
                        Ok(CoreEvent::Restart) => println!("CoreBridge Restart event received."),
                        Ok(CoreEvent::Shutdown) => {
                            println!("CoreBridge Shutdown event received. Stopping...");
                            break;
                        }
                        Err(_) => {
                            println!("CoreBridge: Channel closed. Exiting...");
                            break;
                        }
                    }
                }
            }
        }
    }

    async fn handle_client(
        _socket: TcpStream,
        _addr: String,
        _active_users: Arc<Mutex<HashMap<String, String>>>,
        core_event_tx: UnboundedSender<CoreEvent>,
    ) {
        let command = "STARTUP,RESTART";
        let commands: Vec<&str> = command.split(',').collect();

        for cmd in commands {
            match cmd.trim() {
                "STARTUP" => {
                    let _ = core_event_tx.send(CoreEvent::Startup);
                }
                "RESTART" => {
                    let _ = core_event_tx.send(CoreEvent::Restart);
                }
                "SHUTDOWN" => {
                    let _ = core_event_tx.send(CoreEvent::Shutdown);
                }
                _ => {
                    println!("Unknown command: {}", cmd);
                }
            }
        }
    }

    pub async fn start_grpc_server(core_bridge: Arc<Mutex<Self>>) {
        let addr = "[::1]:50051".parse().unwrap();

        let server = Server::builder()
            .add_service(CoreBridgeServiceServer::new(CoreBridgeGrpcWrapper(
                core_bridge.clone(),
            )))
            .serve_with_shutdown(addr, shutdown_signal(Arc::clone(&core_bridge)));

        println!("gRPC server started at {:?}", addr);

        // Wait for the server to either complete or be shut down
        if let Err(e) = server.await {
            eprintln!("Server error: {}", e);
        }

        println!("gRPC server shut down.");
    }
}

// gRPC-safe wrapper around CoreBridge
#[derive(Clone)]
pub struct CoreBridgeGrpcWrapper(pub Arc<Mutex<CoreBridge>>);

#[tonic::async_trait]
impl CoreBridgeService for CoreBridgeGrpcWrapper {
    type StreamStatusUpdatesStream =
        Pin<Box<dyn Stream<Item = Result<StatusUpdate, tonic::Status>> + Send>>;

    async fn execute_command(
        &self,
        request: Request<CommandRequest>,
    ) -> Result<Response<CommandResponse>, Status> {
        let command_string = request.into_inner().command;
        let inner = self.0.lock().await;
        let response = inner.handle_command_logic(command_string).await;
        Ok(Response::new(response))
    }

    async fn stream_status_updates(
        &self,
        _request: Request<()>,
    ) -> Result<Response<Self::StreamStatusUpdatesStream>, Status> {
        let mut inner = self.0.lock().await;
        let stream = inner.generate_status_stream()?;
        Ok(Response::new(stream))
    }
}

// Shutdown signal that you can call when you need to stop the server
async fn shutdown_signal(core_bridge: Arc<Mutex<CoreBridge>>) {
    signal::ctrl_c().await.unwrap();
    println!("Shutdown signal received. Stopping server...");

    // Lock CoreBridge to send shutdown event through the tx channel
    let inner = core_bridge.lock().await;

    // Send the shutdown event over the channel
    inner.core_event_manip_tx.send(CoreEvent::Shutdown).unwrap();
    println!("Shutdown event sent to CoreBridge.");
}
