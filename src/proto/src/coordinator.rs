#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetDataflowRequest {
    #[prost(message, optional, tag = "1")]
    pub job_id: ::core::option::Option<super::common::ResourceId>,
}
/// Generated client implementations.
pub mod coordinator_api_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// / RPC Api for Coordinator
    #[derive(Debug, Clone)]
    pub struct CoordinatorApiClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl CoordinatorApiClient<tonic::transport::Channel> {
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
    impl<T> CoordinatorApiClient<T>
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
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> CoordinatorApiClient<InterceptedService<T, F>>
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
            CoordinatorApiClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// / Attempt to deploy a new dataflow and create a JobManager.
        /// / Unless bump into network problems, JobManager will be informed the status of the deployed dataflow asynchronously.
        pub async fn create_dataflow(
            &mut self,
            request: impl tonic::IntoRequest<super::super::common::Dataflow>,
        ) -> Result<tonic::Response<super::super::common::Response>, tonic::Status> {
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
                "/coordinator.CoordinatorApi/CreateDataflow",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// / Attempt to terminate a dataflow
        /// / Unless bump into network problems, JobManager will be informed the status of the deployed dataflow asynchronously.
        /// / After the status is transitioned into TERMINATED, the JobManager will be removed from coordinator
        pub async fn terminate_dataflow(
            &mut self,
            request: impl tonic::IntoRequest<super::super::common::ResourceId>,
        ) -> Result<tonic::Response<super::super::common::Response>, tonic::Status> {
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
                "/coordinator.CoordinatorApi/TerminateDataflow",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// / Get the details of a dataflow.
        /// / The details contains: each operator's status, metrics, basic information, checkpoint status, etc.
        pub async fn get_dataflow(
            &mut self,
            request: impl tonic::IntoRequest<super::GetDataflowRequest>,
        ) -> Result<
            tonic::Response<super::super::common::DataflowStates>,
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
                "/coordinator.CoordinatorApi/GetDataflow",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// / Receive ack
        pub async fn receive_ack(
            &mut self,
            request: impl tonic::IntoRequest<super::super::common::Ack>,
        ) -> Result<tonic::Response<super::super::common::Response>, tonic::Status> {
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
                "/coordinator.CoordinatorApi/ReceiveAck",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        /// / Receive heartbeat
        pub async fn receive_heartbeat(
            &mut self,
            request: impl tonic::IntoRequest<super::super::common::Heartbeat>,
        ) -> Result<tonic::Response<super::super::common::Response>, tonic::Status> {
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
                "/coordinator.CoordinatorApi/ReceiveHeartbeat",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod coordinator_api_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with CoordinatorApiServer.
    #[async_trait]
    pub trait CoordinatorApi: Send + Sync + 'static {
        /// / Attempt to deploy a new dataflow and create a JobManager.
        /// / Unless bump into network problems, JobManager will be informed the status of the deployed dataflow asynchronously.
        async fn create_dataflow(
            &self,
            request: tonic::Request<super::super::common::Dataflow>,
        ) -> Result<tonic::Response<super::super::common::Response>, tonic::Status>;
        /// / Attempt to terminate a dataflow
        /// / Unless bump into network problems, JobManager will be informed the status of the deployed dataflow asynchronously.
        /// / After the status is transitioned into TERMINATED, the JobManager will be removed from coordinator
        async fn terminate_dataflow(
            &self,
            request: tonic::Request<super::super::common::ResourceId>,
        ) -> Result<tonic::Response<super::super::common::Response>, tonic::Status>;
        /// / Get the details of a dataflow.
        /// / The details contains: each operator's status, metrics, basic information, checkpoint status, etc.
        async fn get_dataflow(
            &self,
            request: tonic::Request<super::GetDataflowRequest>,
        ) -> Result<
            tonic::Response<super::super::common::DataflowStates>,
            tonic::Status,
        >;
        /// / Receive ack
        async fn receive_ack(
            &self,
            request: tonic::Request<super::super::common::Ack>,
        ) -> Result<tonic::Response<super::super::common::Response>, tonic::Status>;
        /// / Receive heartbeat
        async fn receive_heartbeat(
            &self,
            request: tonic::Request<super::super::common::Heartbeat>,
        ) -> Result<tonic::Response<super::super::common::Response>, tonic::Status>;
    }
    /// / RPC Api for Coordinator
    #[derive(Debug)]
    pub struct CoordinatorApiServer<T: CoordinatorApi> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: CoordinatorApi> CoordinatorApiServer<T> {
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
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for CoordinatorApiServer<T>
    where
        T: CoordinatorApi,
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
                "/coordinator.CoordinatorApi/CreateDataflow" => {
                    #[allow(non_camel_case_types)]
                    struct CreateDataflowSvc<T: CoordinatorApi>(pub Arc<T>);
                    impl<
                        T: CoordinatorApi,
                    > tonic::server::UnaryService<super::super::common::Dataflow>
                    for CreateDataflowSvc<T> {
                        type Response = super::super::common::Response;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::common::Dataflow>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).create_dataflow(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateDataflowSvc(inner);
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
                "/coordinator.CoordinatorApi/TerminateDataflow" => {
                    #[allow(non_camel_case_types)]
                    struct TerminateDataflowSvc<T: CoordinatorApi>(pub Arc<T>);
                    impl<
                        T: CoordinatorApi,
                    > tonic::server::UnaryService<super::super::common::ResourceId>
                    for TerminateDataflowSvc<T> {
                        type Response = super::super::common::Response;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::common::ResourceId>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).terminate_dataflow(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TerminateDataflowSvc(inner);
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
                "/coordinator.CoordinatorApi/GetDataflow" => {
                    #[allow(non_camel_case_types)]
                    struct GetDataflowSvc<T: CoordinatorApi>(pub Arc<T>);
                    impl<
                        T: CoordinatorApi,
                    > tonic::server::UnaryService<super::GetDataflowRequest>
                    for GetDataflowSvc<T> {
                        type Response = super::super::common::DataflowStates;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetDataflowRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).get_dataflow(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetDataflowSvc(inner);
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
                "/coordinator.CoordinatorApi/ReceiveAck" => {
                    #[allow(non_camel_case_types)]
                    struct ReceiveAckSvc<T: CoordinatorApi>(pub Arc<T>);
                    impl<
                        T: CoordinatorApi,
                    > tonic::server::UnaryService<super::super::common::Ack>
                    for ReceiveAckSvc<T> {
                        type Response = super::super::common::Response;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::common::Ack>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).receive_ack(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ReceiveAckSvc(inner);
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
                "/coordinator.CoordinatorApi/ReceiveHeartbeat" => {
                    #[allow(non_camel_case_types)]
                    struct ReceiveHeartbeatSvc<T: CoordinatorApi>(pub Arc<T>);
                    impl<
                        T: CoordinatorApi,
                    > tonic::server::UnaryService<super::super::common::Heartbeat>
                    for ReceiveHeartbeatSvc<T> {
                        type Response = super::super::common::Response;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::common::Heartbeat>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).receive_heartbeat(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ReceiveHeartbeatSvc(inner);
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
    impl<T: CoordinatorApi> Clone for CoordinatorApiServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: CoordinatorApi> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: CoordinatorApi> tonic::server::NamedService for CoordinatorApiServer<T> {
        const NAME: &'static str = "coordinator.CoordinatorApi";
    }
}
