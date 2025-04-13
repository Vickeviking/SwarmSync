/// ---- Commands from AdminShell to CoreBridge ----
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommandRequest {
    /// Command name (e.g., Startup, Restart, Shutdown)
    #[prost(string, tag="1")]
    pub command: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommandResponse {
    /// Status of the command (e.g., success, failure)
    #[prost(string, tag="1")]
    pub status: ::prost::alloc::string::String,
    /// Result message or additional info
    #[prost(string, tag="2")]
    pub result: ::prost::alloc::string::String,
}
/// ---- Status updates from CoreBridge to AdminShell ----
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StatusUpdate {
    /// E.g., "heartbeat", "task", etc.
    #[prost(string, tag="1")]
    pub update_type: ::prost::alloc::string::String,
    /// Heartbeat integer or other status value
    #[prost(int32, tag="2")]
    pub value: i32,
}
/// Generated client implementations.
pub mod core_bridge_service_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct CoreBridgeServiceClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl CoreBridgeServiceClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> CoreBridgeServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> CoreBridgeServiceClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            CoreBridgeServiceClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        /// AdminShell sends commands, CoreBridge performs them and responds
        pub async fn execute_command(
            &mut self,
            request: impl tonic::IntoRequest<super::CommandRequest>,
        ) -> Result<tonic::Response<super::CommandResponse>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/swarm_sync.CoreBridgeService/ExecuteCommand",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// CoreBridge streams status updates to AdminShell
        pub async fn stream_status_updates(
            &mut self,
            request: impl tonic::IntoRequest<()>,
        ) -> Result<
            tonic::Response<tonic::codec::Streaming<super::StatusUpdate>>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/swarm_sync.CoreBridgeService/StreamStatusUpdates",
            );
            self.inner.server_streaming(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod core_bridge_service_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with CoreBridgeServiceServer.
    #[async_trait]
    pub trait CoreBridgeService: Send + Sync + 'static {
        /// AdminShell sends commands, CoreBridge performs them and responds
        async fn execute_command(
            &self,
            request: tonic::Request<super::CommandRequest>,
        ) -> Result<tonic::Response<super::CommandResponse>, tonic::Status>;
        ///Server streaming response type for the StreamStatusUpdates method.
        type StreamStatusUpdatesStream: futures_core::Stream<
                Item = Result<super::StatusUpdate, tonic::Status>,
            >
            + Send
            + 'static;
        /// CoreBridge streams status updates to AdminShell
        async fn stream_status_updates(
            &self,
            request: tonic::Request<()>,
        ) -> Result<tonic::Response<Self::StreamStatusUpdatesStream>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct CoreBridgeServiceServer<T: CoreBridgeService> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: CoreBridgeService> CoreBridgeServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for CoreBridgeServiceServer<T>
    where
        T: CoreBridgeService,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/swarm_sync.CoreBridgeService/ExecuteCommand" => {
                    #[allow(non_camel_case_types)]
                    struct ExecuteCommandSvc<T: CoreBridgeService>(pub Arc<T>);
                    impl<
                        T: CoreBridgeService,
                    > tonic::server::UnaryService<super::CommandRequest>
                    for ExecuteCommandSvc<T> {
                        type Response = super::CommandResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CommandRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).execute_command(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ExecuteCommandSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/swarm_sync.CoreBridgeService/StreamStatusUpdates" => {
                    #[allow(non_camel_case_types)]
                    struct StreamStatusUpdatesSvc<T: CoreBridgeService>(pub Arc<T>);
                    impl<T: CoreBridgeService> tonic::server::ServerStreamingService<()>
                    for StreamStatusUpdatesSvc<T> {
                        type Response = super::StatusUpdate;
                        type ResponseStream = T::StreamStatusUpdatesStream;
                        type Future = BoxFuture<
                            tonic::Response<Self::ResponseStream>,
                            tonic::Status,
                        >;
                        fn call(&mut self, request: tonic::Request<()>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).stream_status_updates(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StreamStatusUpdatesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.server_streaming(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: CoreBridgeService> Clone for CoreBridgeServiceServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: CoreBridgeService> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: CoreBridgeService> tonic::transport::NamedService
    for CoreBridgeServiceServer<T> {
        const NAME: &'static str = "swarm_sync.CoreBridgeService";
    }
}
