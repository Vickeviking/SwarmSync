use crate::enums::system::{CoreEvent, Pulse};
use crate::generated::{
    core_bridge_service_server::CoreBridgeService,
    core_bridge_service_server::CoreBridgeServiceServer, CommandRequest, CommandResponse,
    StatusUpdate,
};
use crate::pulse_broadcaster::PulseBroadcaster;
use crate::service_channels::{self, ChannelType, EventPayload};
use crate::shared_resources::SharedResources;
use futures::stream::Stream;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::{broadcast::Receiver, mpsc, mpsc::UnboundedSender, Mutex, Notify};
use tokio_stream::{wrappers, wrappers::BroadcastStream, StreamExt};
use tonic::{transport::Server, Request, Response, Status};

pub type StatusUpdateStream = Pin<Box<dyn Stream<Item = Result<StatusUpdate, Status>> + Send>>;

// CoreBridge main struct
pub struct CoreBridge {
    shared_resources: Arc<SharedResources>,
    core_bridge_to_main_core_event_tx: Option<UnboundedSender<EventPayload>>,
}

// CoreBridge constructor
impl CoreBridge {
    pub fn new(shared_resources: Arc<SharedResources>) -> Self {
        CoreBridge {
            shared_resources,
            core_bridge_to_main_core_event_tx: None,
        }
    }

    pub async fn handle_command_logic(
        &self,
        command_string: String,
        notify: Arc<Notify>,
    ) -> CommandResponse {
        let commands: Vec<&str> = command_string.split(',').collect();

        for command in commands {
            match command.trim() {
                "STARTUP" => {
                    println!("Received STARTUP command");
                    if let Some(tx) = &self.core_bridge_to_main_core_event_tx {
                        let _ = tx.send(EventPayload::CoreEvent(CoreEvent::Startup));
                    } else {
                        return CommandResponse {
                            status: "error".into(),
                            result: "core_bridge_to_main_core_event_tx not bound in CoreBridge"
                                .into(),
                        };
                    }
                }
                "RESTART" => {
                    println!("Received RESTART command");
                    if let Some(tx) = &self.core_bridge_to_main_core_event_tx {
                        let _ = tx.send(EventPayload::CoreEvent(CoreEvent::Restart));
                    } else {
                        return CommandResponse {
                            status: "error".into(),
                            result: "core_bridge_to_main_core_event_tx not bound in CoreBridge"
                                .into(),
                        };
                    }
                }
                "SHUTDOWN" => {
                    println!("Received SHUTDOWN command");
                    if let Some(tx) = &self.core_bridge_to_main_core_event_tx {
                        let _ = tx.send(EventPayload::CoreEvent(CoreEvent::Shutdown));
                    } else {
                        return CommandResponse {
                            status: "error".into(),
                            result: "core_bridge_to_main_core_event_tx not bound in CoreBridge"
                                .into(),
                        };
                    }
                    // Trigger shutdown notify
                    notify.notify_one();
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
        // Take the receiver and ensure it's valid
        let rx = self
            .shared_resources
            .get_pulse_subscriptions()
            .subscribe_medium();

        // Create a new stream from the receiver
        let stream = BroadcastStream::new(rx).filter_map(|result| match result {
            Ok(_) | Err(wrappers::errors::BroadcastStreamRecvError::Lagged(_)) => {
                Some(Ok(StatusUpdate {
                    update_type: "heartbeat".to_string(),
                    value: 1,
                }))
            }
        });

        // Return the stream wrapped in a Box
        Ok(Box::pin(stream))
    }

    pub async fn wire_channels(mut self) -> Result<Self, String> {
        // Critical section, we take core_bridge_to_main_core_event_tx here!
        {
            let service_wiring = self.shared_resources.get_service_wiring();
            let locked_service_wiring = service_wiring.lock().await;
            self.core_bridge_to_main_core_event_tx = locked_service_wiring
                .take_tx(ChannelType::CoreBridgeToMain_CoreEvents)
                .await;

            //check that core_bridge_to_main_core_event_tx is Some, otherwise throw error
            if self.core_bridge_to_main_core_event_tx.is_none() {
                return Err(
                    "CoreBridge wiring failed: Missing CoreBridgeToMain_CoreEvents TX".into(),
                );
            }
        }
        Ok(self)
    }

    pub async fn init(mut core_event_rx: Receiver<CoreEvent>, notify: Arc<Notify>) {
        // Main loop to listen for events
        loop {
            tokio::select! {
                event = core_event_rx.recv() => {
                    match event {
                        Ok(CoreEvent::Startup) => println!("CoreBridge Startup event received."),
                        Ok(CoreEvent::Restart) => println!("CoreBridge Restart event received."),
                        Ok(CoreEvent::Shutdown) => {
                            notify.notify_one();
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

    pub async fn start_grpc_server(core_bridge: Arc<Mutex<Self>>, notify: Arc<Notify>) {
        let addr = "0.0.0.0:50052".parse().unwrap();

        let server = Server::builder()
            .add_service(CoreBridgeServiceServer::new(CoreBridgeGrpcWrapper(
                core_bridge.clone(),
                notify.clone(),
            )))
            .serve_with_shutdown(addr, shutdown_signal(notify.clone()));

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
pub struct CoreBridgeGrpcWrapper(pub Arc<Mutex<CoreBridge>>, pub Arc<Notify>);

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
        let response = inner
            .handle_command_logic(command_string, self.1.clone())
            .await;
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

async fn shutdown_signal(notify: Arc<Notify>) {
    notify.notified().await; // Wait for the shutdown notification
    println!("Shutdown signal received. Stopping gRPC server...");
}
