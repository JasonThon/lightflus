#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DispatchDataEventsRequest {
    #[prost(message, repeated, tag = "1")]
    pub events: ::prost::alloc::vec::Vec<super::common::KeyedDataEvent>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DispatchDataEventsResponse {
    #[prost(map = "string, enumeration(DispatchDataEventStatusEnum)", tag = "1")]
    pub status_set: ::std::collections::HashMap<::prost::alloc::string::String, i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StopDataflowRequest {
    #[prost(message, optional, tag = "1")]
    pub job_id: ::core::option::Option<super::common::ResourceId>,
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
pub enum DispatchDataEventStatusEnum {
    Dispatching = 0,
    Done = 1,
    Failure = 2,
}
impl DispatchDataEventStatusEnum {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            DispatchDataEventStatusEnum::Dispatching => "DISPATCHING",
            DispatchDataEventStatusEnum::Done => "DONE",
            DispatchDataEventStatusEnum::Failure => "FAILURE",
        }
    }
}
/// Generated client implementations.
pub mod task_worker_api_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct TaskWorkerApiClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl TaskWorkerApiClient<tonic::transport::Channel> {
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
    impl<T> TaskWorkerApiClient<T>
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
        ) -> TaskWorkerApiClient<InterceptedService<T, F>>
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
            TaskWorkerApiClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn probe(
            &mut self,
            request: impl tonic::IntoRequest<super::super::common::ProbeRequest>,
        ) -> Result<
            tonic::Response<super::super::common::ProbeResponse>,
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
                "/worker.TaskWorkerApi/Probe",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn dispatch_data_events(
            &mut self,
            request: impl tonic::IntoRequest<super::DispatchDataEventsRequest>,
        ) -> Result<tonic::Response<super::DispatchDataEventsResponse>, tonic::Status> {
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
                "/worker.TaskWorkerApi/DispatchDataEvents",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn stop_dataflow(
            &mut self,
            request: impl tonic::IntoRequest<super::StopDataflowRequest>,
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
                "/worker.TaskWorkerApi/StopDataflow",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
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
                "/worker.TaskWorkerApi/CreateSubDataflow",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod task_worker_api_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with TaskWorkerApiServer.
    #[async_trait]
    pub trait TaskWorkerApi: Send + Sync + 'static {
        async fn probe(
            &self,
            request: tonic::Request<super::super::common::ProbeRequest>,
        ) -> Result<tonic::Response<super::super::common::ProbeResponse>, tonic::Status>;
        async fn dispatch_data_events(
            &self,
            request: tonic::Request<super::DispatchDataEventsRequest>,
        ) -> Result<tonic::Response<super::DispatchDataEventsResponse>, tonic::Status>;
        async fn stop_dataflow(
            &self,
            request: tonic::Request<super::StopDataflowRequest>,
        ) -> Result<tonic::Response<super::StopDataflowResponse>, tonic::Status>;
        async fn create_sub_dataflow(
            &self,
            request: tonic::Request<super::CreateSubDataflowRequest>,
        ) -> Result<tonic::Response<super::CreateSubDataflowResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct TaskWorkerApiServer<T: TaskWorkerApi> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: TaskWorkerApi> TaskWorkerApiServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for TaskWorkerApiServer<T>
    where
        T: TaskWorkerApi,
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
                "/worker.TaskWorkerApi/Probe" => {
                    #[allow(non_camel_case_types)]
                    struct ProbeSvc<T: TaskWorkerApi>(pub Arc<T>);
                    impl<
                        T: TaskWorkerApi,
                    > tonic::server::UnaryService<super::super::common::ProbeRequest>
                    for ProbeSvc<T> {
                        type Response = super::super::common::ProbeResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::super::common::ProbeRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).probe(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ProbeSvc(inner);
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
                "/worker.TaskWorkerApi/DispatchDataEvents" => {
                    #[allow(non_camel_case_types)]
                    struct DispatchDataEventsSvc<T: TaskWorkerApi>(pub Arc<T>);
                    impl<
                        T: TaskWorkerApi,
                    > tonic::server::UnaryService<super::DispatchDataEventsRequest>
                    for DispatchDataEventsSvc<T> {
                        type Response = super::DispatchDataEventsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DispatchDataEventsRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).dispatch_data_events(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DispatchDataEventsSvc(inner);
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
                "/worker.TaskWorkerApi/StopDataflow" => {
                    #[allow(non_camel_case_types)]
                    struct StopDataflowSvc<T: TaskWorkerApi>(pub Arc<T>);
                    impl<
                        T: TaskWorkerApi,
                    > tonic::server::UnaryService<super::StopDataflowRequest>
                    for StopDataflowSvc<T> {
                        type Response = super::StopDataflowResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::StopDataflowRequest>,
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
                "/worker.TaskWorkerApi/CreateSubDataflow" => {
                    #[allow(non_camel_case_types)]
                    struct CreateSubDataflowSvc<T: TaskWorkerApi>(pub Arc<T>);
                    impl<
                        T: TaskWorkerApi,
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
    impl<T: TaskWorkerApi> Clone for TaskWorkerApiServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: TaskWorkerApi> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: TaskWorkerApi> tonic::server::NamedService for TaskWorkerApiServer<T> {
        const NAME: &'static str = "worker.TaskWorkerApi";
    }
}
