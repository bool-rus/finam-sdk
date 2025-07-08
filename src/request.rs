use tonic::client::Grpc;
use tonic::service::interceptor::InterceptedService;
use tonic::transport::Channel;

use crate::auth::TokenInterceptor;
use crate::GrpcInner;
use crate::proto::{
    auth::{auth_service_client::AuthServiceClient, *},
    accounts::{accounts_service_client::AccountsServiceClient, *},
    assets::{assets_service_client::AssetsServiceClient, *},
    marketdata::{market_data_service_client::MarketDataServiceClient, *},
    orders::{orders_service_client::OrdersServiceClient, *},
};


pub trait Requestor<Req, Res> {
    fn req(self, req: Req) -> impl Future<Output=Result<Res, tonic::Status>> + Send;
}

macro_rules! requestor_impl {
    ($($res:ty = $client:ident : $method:ident($req:ty),)+) => {$(
        impl<T> Requestor<$req, $res> for Grpc<T> where T: GrpcInner + Send + 'static {
            fn req(self, req: $req) -> impl Future<Output=Result<$res, tonic::Status>> + Send {Box::pin(async move {
                let mut client = $client::from(self);
                Ok(client.$method(req).await?.into_inner())
            })}
        }
    )+}
}

requestor_impl![
    AuthResponse = AuthServiceClient: auth(AuthRequest),
    TokenDetailsResponse = AuthServiceClient: token_details(TokenDetailsRequest),
    GetAccountResponse = AccountsServiceClient: get_account(GetAccountRequest),
    TradesResponse = AccountsServiceClient: trades(TradesRequest),
    TransactionsResponse = AccountsServiceClient: transactions(TransactionsRequest),
    ExchangesResponse = AssetsServiceClient: exchanges(ExchangesRequest),
    AssetsResponse = AssetsServiceClient: assets(AssetsRequest),
    GetAssetResponse = AssetsServiceClient: get_asset(GetAssetRequest),
    GetAssetParamsResponse = AssetsServiceClient: get_asset_params(GetAssetParamsRequest),
    OptionsChainResponse = AssetsServiceClient: options_chain(OptionsChainRequest),
    ScheduleResponse = AssetsServiceClient: schedule(ScheduleRequest),
    ClockResponse = AssetsServiceClient: clock(ClockRequest),
    BarsResponse = MarketDataServiceClient: bars(BarsRequest),  
    QuoteResponse = MarketDataServiceClient: last_quote(QuoteRequest),  
    OrderBookResponse = MarketDataServiceClient: order_book(OrderBookRequest),  
    LatestTradesResponse = MarketDataServiceClient: latest_trades(LatestTradesRequest),  
    OrderState = OrdersServiceClient: place_order(Order),  
    OrderState = OrdersServiceClient: cancel_order(CancelOrderRequest),  
    OrderState = OrdersServiceClient: get_order(GetOrderRequest),  
    OrdersResponse = OrdersServiceClient: get_orders(OrdersRequest),  
];

impl<Req, Res> Requestor<Req,Res> for &crate::FinamApi where Grpc<InterceptedService<Channel, TokenInterceptor>>: Requestor<Req, Res> {
    fn req(self, req: Req) -> impl Future<Output=Result<Res, tonic::Status>> + Send {
        let grpc = self.grpc();
        grpc.req(req)
    }
}