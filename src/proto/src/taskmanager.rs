#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendEventToOperatorResponse {
    #[prost(enumeration = "SendEventToOperatorStatusEnum", tag = "1")]
    pub status: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StopDataflowResponse {
    #[prost(message, optional, tag = "1")]
    pub resp: ::core::option::Option<super::common::Response>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateSubDataflowRequest {
    #[prost(message, optional, tag = "1")]
    pub job_id: ::core::option::Option<super::common::ResourceId>,
    #[prost(message, optional, tag = "2")]
    pub dataflow: ::core::option::Option<super::common::Dataflow>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateSubDataflowResponse {
    #[prost(enumeration = "super::common::DataflowStatus", tag = "1")]
    pub status: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SendEventToOperatorStatusEnum {
    Dispatching = 0,
    Done = 1,
    Failure = 2,
}
impl SendEventToOperatorStatusEnum {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SendEventToOperatorStatusEnum::Dispatching => "DISPATCHING",
            SendEventToOperatorStatusEnum::Done => "DONE",
            SendEventToOperatorStatusEnum::Failure => "FAILURE",
        }
    }
}
/// Generated client implementations.
pub mod task_manager_api_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[doc = "/ RPC Api for Task Manager"]
    #[derive(Debug, Clone)]
    pub struct TaskManagerApiClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl TaskManagerApiClient<tonic::transport::Channel> {
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
    impl<T> TaskManagerApiClient<T>
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
        ) -> TaskManagerApiClient<InterceptedService<T, F>>
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
            TaskManagerApiClient::new(InterceptedService::new(inner, interceptor))
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
        #[doc = "/ Send event to operator"]
        pub async fn send_event_to_operator(
            &mut self,
            request: impl tonic::IntoRequest<super::super::common::KeyedDataEvent>,
        ) -> Result<tonic::Response<super::SendEventToOperatorResponse>, tonic::Status> {
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
                "/taskmanager.TaskManagerApi/SendEventToOperator",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = "/ Attempt to terminate a sub-dataflow"]
        pub async fn stop_dataflow(
            &mut self,
            request: impl tonic::IntoRequest<super::super::common::ResourceId>,
        ) -> Result<tonic::Response<super::StopDataflowResponse>, tonic::Status> {
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
                "/taskmanager.TaskManagerApi/StopDataflow",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = "/ Attempt to create a sub-dataflow"]
        pub async fn create_sub_dataflow(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateSubDataflowRequest>,
        ) -> Result<tonic::Response<super::CreateSubDataflowResponse>, tonic::Status> {
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
                "/taskmanager.TaskManagerApi/CreateSubDataflow",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = "/ Receive heartbeat"]
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
                "/taskmanager.TaskManagerApi/ReceiveHeartbeat",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        #[doc = "/ Receive ack"]
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
                "/taskmanager.TaskManagerApi/ReceiveAck",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod task_manager_api_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with TaskManagerApiServer.
    #[async_trait]
    pub trait TaskManagerApi: Send + Sync + 'static {
        #[doc = "/ Send event to operator"]
        async fn send_event_to_operator(
            &self,
            request: tonic::Request<super::super::common::KeyedDataEvent>,
        ) -> Result<tonic::Response<super::SendEventToOperatorResponse>, tonic::Status>;
        #[doc = "/ Attempt to terminate a sub-dataflow"]
        async fn stop_dataflow(
            &self,
            request: tonic::Request<super::super::common::ResourceId>,
        ) -> Result<tonic::Response<super::StopDataflowResponse>, tonic::Status>;
        #[doc = "/ Attempt to create a sub-dataflow"]
        async fn create_sub_dataflow(
            &self,
            request: tonic::Request<super::CreateSubDataflowRequest>,
        ) -> Result<tonic::Response<super::CreateSubDataflowResponse>, tonic::Status>;
        #[doc = "/ Receive heartbeat"]
        async fn receive_heartbeat(
            &self,
            request: tonic::Request<super::super::common::Heartbeat>,
        ) -> Result<tonic::Response<super::super::common::Response>, tonic::Status>;
        #[doc = "/ Receive ack"]
        async fn receive_ack(
            &self,
            request: tonic::Request<super::super::common::Ack>,
        ) -> Result<tonic::Response<super::super::common::Response>, tonic::Status>;
    }
    #[doc = "/ RPC Api for Task Manager"]
    #[derive(Debug)]
    pub struct TaskManagerApiServer<T: TaskManagerApi> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: TaskManagerApi> TaskManagerApiServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for TaskManagerApiServer<T>
    where
        T: TaskManagerApi,
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
                "/taskmanager.TaskManagerApi/SendEventToOperator" => {
                    #[allow(non_camel_case_types)]
                    struct SendEventToOperatorSvc<T: TaskManagerApi>(pub Arc<T>);
                    impl<
                        T: TaskManagerApi,
                    > tonic::server::UnaryService<super::super::common::KeyedDataEvent>
                    for SendEventToOperatorSvc<T> {
                        type Response = super::SendEventToOperatorResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::common::KeyedDataEvent>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).send_event_to_operator(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SendEventToOperatorSvc(inner);
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
                "/taskmanager.TaskManagerApi/StopDataflow" => {
                    #[allow(non_camel_case_types)]
                    struct StopDataflowSvc<T: TaskManagerApi>(pub Arc<T>);
                    impl<
                        T: TaskManagerApi,
                    > tonic::server::UnaryService<super::super::common::ResourceId>
                    for StopDataflowSvc<T> {
                        type Response = super::StopDataflowResponse;
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
                                (*inner).stop_dataflow(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = StopDataflowSvc(inner);
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
                "/taskmanager.TaskManagerApi/CreateSubDataflow" => {
                    #[allow(non_camel_case_types)]
                    struct CreateSubDataflowSvc<T: TaskManagerApi>(pub Arc<T>);
                    impl<
                        T: TaskManagerApi,
                    > tonic::server::UnaryService<super::CreateSubDataflowRequest>
                    for CreateSubDataflowSvc<T> {
                        type Response = super::CreateSubDataflowResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateSubDataflowRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).create_sub_dataflow(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = CreateSubDataflowSvc(inner);
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
                "/taskmanager.TaskManagerApi/ReceiveHeartbeat" => {
                    #[allow(non_camel_case_types)]
                    struct ReceiveHeartbeatSvc<T: TaskManagerApi>(pub Arc<T>);
                    impl<
                        T: TaskManagerApi,
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
                "/taskmanager.TaskManagerApi/ReceiveAck" => {
                    #[allow(non_camel_case_types)]
                    struct ReceiveAckSvc<T: TaskManagerApi>(pub Arc<T>);
                    impl<
                        T: TaskManagerApi,
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
    impl<T: TaskManagerApi> Clone for TaskManagerApiServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: TaskManagerApi> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: TaskManagerApi> tonic::server::NamedService for TaskManagerApiServer<T> {
        const NAME: &'static str = "taskmanager.TaskManagerApi";
    }
}
