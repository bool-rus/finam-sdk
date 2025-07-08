use futures::Sink;
use futures::SinkExt;
use tonic::client::Grpc;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;
use std::future::Future;
use tokio::task::JoinHandle;
use crate::proto::marketdata::{market_data_service_client::MarketDataServiceClient, *};
use crate::proto::orders::orders_service_client::OrdersServiceClient;
use crate::proto::orders::OrderTradeRequest;
use crate::proto::orders::OrderTradeResponse;
use crate::FinamApi;


pub trait StartStream<Req, Res> {
    fn start_stream(self, req: Req, sink: impl Sink<Res> + Unpin + Send + 'static) -> impl Future<Output = Result<JoinHandle<()>, tonic::Status>> + Send;
}

impl<T> StartStream<OrderTradeRequest, OrderTradeResponse> for Grpc<T> where T: crate::GrpcInner + Send + 'static {
    fn start_stream(self, req: OrderTradeRequest, mut sink: impl Sink<OrderTradeResponse> + Unpin + Send + 'static) 
        -> impl Future<Output = Result<JoinHandle<()>, tonic::Status>> + Send { Box::pin(async move {
        let mut serv = OrdersServiceClient::from(self);
        let (mut sender, receiver) = futures::channel::mpsc::unbounded();
        sender.send(req.clone()).await.unwrap();
        let res = serv.subscribe_order_trade(receiver).await;
        let mut stream: tonic::codec::Streaming<OrderTradeResponse> = res?.into_inner();
        log::info!("Subscribed by req: {req:?}");
        let handle = tokio::spawn(async move {loop {
            match stream.message().await {
                Ok(Some(msg)) => {
                    if sink.send(msg).await.is_err() {
                        log::error!("Subscribtion to {req:?} cancelled");
                        break;
                    }
                    continue;
                }
                Err(e) => log::error!("{e}"),
                Ok(None) => {},
            };
            let (mut sender, receiver) = futures::channel::mpsc::unbounded();
            sender.send(req.clone()).await.unwrap();
            match serv.subscribe_order_trade(receiver).await {
                Ok(response) => {
                    stream = response.into_inner();
                    log::info!("Successfull resubscribed: {req:?}")
                },
                Err(e) => {
                    log::error!("Cannot resubscribe to {req:?}: {e}");
                    tokio::time::sleep(crate::RETRY_DELAY).await;
                }
            }
        }});
        Ok(handle)
    })}
}

macro_rules! start_stream_impl {
    ($($res:ty = $client:ident : $method:ident($req:ty),)+) => {$(
        impl<T> StartStream<$req,$res> for Grpc<T> where T: crate::GrpcInner + Send + 'static {
            fn start_stream(self, req: $req, mut sink: impl Sink<$res> + Unpin + Send + 'static) 
                -> impl Future<Output = Result<JoinHandle<()>, tonic::Status>> + Send { Box::pin(async move {
                let mut serv = $client::from(self);
                let mut stream: tonic::codec::Streaming<$res> = serv.$method(req.clone()).await?.into_inner();
                log::info!("Subscribed by req: {req:?}");
                let handle = tokio::spawn(async move {loop {
                    match stream.message().await {
                        Ok(Some(msg)) => {
                            if sink.send(msg).await.is_err() {
                                log::error!("Subscribtion to {req:?} cancelled");
                                break;
                            }
                            continue;
                        }
                        Err(e) => log::error!("{e}"),
                        Ok(None) => {},
                    };
                    match serv.$method(req.clone()).await {
                        Ok(response) => {
                            stream = response.into_inner();
                            log::info!("Successfull resubscribed: {req:?}")
                        },
                        Err(e) => {
                            log::error!("Cannot resubscribe to {req:?}: {e}");
                            tokio::time::sleep(crate::RETRY_DELAY).await;
                        }
                    }
                }});
                Ok(handle)
            })}
        }
    )+}
}
start_stream_impl![
    SubscribeLatestTradesResponse = MarketDataServiceClient:subscribe_latest_trades(SubscribeLatestTradesRequest),
    SubscribeOrderBookResponse = MarketDataServiceClient:subscribe_order_book(SubscribeOrderBookRequest),
    SubscribeQuoteResponse = MarketDataServiceClient:subscribe_quote(SubscribeQuoteRequest),
    SubscribeBarsResponse = MarketDataServiceClient:subscribe_bars(SubscribeBarsRequest),
];



impl<Req, Res> StartStream<Req, Res> for &FinamApi where Grpc<InterceptedService<Channel, crate::auth::TokenInterceptor>>: StartStream<Req, Res> {
    fn start_stream(self, req: Req, sink: impl Sink<Res> + Unpin + Send + 'static) -> impl Future<Output = Result<JoinHandle<()>, tonic::Status>> + Send {
        let grpc = self.grpc();
        grpc.start_stream(req, sink)
    }
}