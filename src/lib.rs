use tonic::{client::Grpc, service::interceptor::InterceptedService, transport::{Channel, ClientTlsConfig}};

use crate::{auth::TokenInterceptor, stream::StartStream};

pub const RETRY_DELAY: std::time::Duration = std::time::Duration::from_secs(5);
pub const FINAM_ENDPOINT: &str = "https://api.finam.ru";

pub mod proto;
pub mod auth;
pub mod stream;
pub mod request;

pub type StdError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub trait GrpcInner: tonic::client::GrpcService<
    tonic::body::Body, 
    Error: Into<StdError> + Send, 
    ResponseBody: tonic::transport::Body<Data = prost::bytes::Bytes, Error: Into<StdError> + Send> + Send  + 'static,
    Future: Send
> {}

impl <B, BE, E, S> GrpcInner for S where 
    S: tonic::client::GrpcService<tonic::body::Body, Error = E, ResponseBody = B>, 
    S::Future: Send,
    E: Into<StdError> + Send, 
    B: tonic::transport::Body<Data = prost::bytes::Bytes, Error = BE> + Send  + 'static,
    BE: Into<StdError> + Send
{}

#[derive(Clone)]
pub struct FinamApi {
    token: std::sync::Arc<std::sync::Mutex<auth::Token>>,
    grpc: Grpc<InterceptedService<Channel, auth::TokenInterceptor>>,
}

impl FinamApi {
    pub async fn connect(secret: String) -> Result<Self, tonic::Status> {
        let tls = ClientTlsConfig::new().with_native_roots();
        let channel = Channel::from_static(FINAM_ENDPOINT).tls_config(tls).unwrap().connect_lazy();
        let interceptor = TokenInterceptor::new(channel.clone(), secret).await?;
        let token = interceptor.get_token();
        let service = Grpc::new(InterceptedService::new(channel, interceptor));
        Ok(Self{token, grpc: service})
    }
    pub fn token(&self) -> String {
        self.token.lock().unwrap().clone().to_str().unwrap().to_string()
    }
    pub fn grpc(&self) -> Grpc<InterceptedService<Channel, auth::TokenInterceptor>> {
        self.grpc.clone()
    }
}
