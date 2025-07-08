use std::{env, error::Error, io::{stdout, Write}, time::Duration};

use futures::StreamExt;
use tokio::time::sleep;
use tonic::{client::Grpc, service::{interceptor::InterceptedService, Interceptor}, transport::{Channel, ClientTlsConfig}, Request};

use finam::{auth::TokenInterceptor, proto::{accounts::{accounts_service_client::AccountsServiceClient, GetAccountRequest}, assets::{assets_service_client::AssetsServiceClient, AssetsRequest, AssetsResponse}, auth::{auth_service_client::AuthServiceClient, AuthResponse, TokenDetailsRequest, TokenDetailsResponse}, google::r#type::Interval, marketdata::{market_data_service_client::MarketDataServiceClient, order_book::row::Action, SubscribeLatestTradesRequest, SubscribeOrderBookRequest}, orders::{order_trade_request, orders_service_client::OrdersServiceClient, OrderTradeRequest}}, request::Requestor, stream::StartStream, FinamApi, RETRY_DELAY};




#[tokio::main]
async fn main() -> Result<(),Box<dyn Error>> {
    simple_logger::SimpleLogger::new().with_colors(true).with_local_timestamps().with_level(log::LevelFilter::Info).init().unwrap();
    go().await?;
    for i in 0..200 {
        sleep(RETRY_DELAY).await;
        print!(".");
        stdout().flush()?;
    }
    log::info!("bye");
    Ok(())
}
async fn go() -> Result<(),Box<dyn Error>> {
    log::info!("started");
    let secret = env::var("TOKEN").expect("Need token");
    let api = FinamApi::connect(secret.clone()).await?;
    let details = api.req(TokenDetailsRequest{token: api.token()}).await?;
    
    let account_id = details.account_ids.first().unwrap().clone();
    println!("{details:#?}");
    let account_id = account_id.clone();
    let a = api.req(GetAccountRequest{ account_id: account_id.clone() }).await?;
    log::info!("{a:#?}");
    
    let symbol = "IMOEXF@RTSX".to_string();
    //let symbol = "SBER@RUSX".to_string();
    //let symbol = "TATN@RUSX".to_string();


    let (sink, mut stream) = futures::channel::mpsc::unbounded();
    //channel.clone().start_stream(SubscribeLatestTradesRequest{ symbol }, sink).await?;
    api.start_stream(OrderTradeRequest{ 
        action: order_trade_request::Action::Subscribe.into(), 
        data_type: 0, 
        account_id: account_id.clone(),
    }, sink).await?;

    //let mut stream = market.subscribe_latest_trades(SubscribeLatestTradesRequest{ symbol }).await?.into_inner();

    while let Some(msg) = stream.next().await {
        log::info!("orders: {:?}", msg.orders.iter().map(|o|format!("{:?}, {:?}", o.status(), o.order)  ).collect::<Vec<_>>());
        log::info!("trades: {:?}", msg.trades.iter().map(|t|format!("{} {:?} {:?} {:?}", t.trade_id, t.side(), t.size.clone().unwrap_or_default(), t.price.clone().unwrap_or_default())).collect::<Vec<_>>());
        break;
    }
    //println!("Hello, world! token: {token}");
    Ok(())
}

fn proc_action(a: finam::proto::marketdata::stream_order_book::row::Action) -> Option<&'static str> {
    Some(match a {
        finam::proto::marketdata::stream_order_book::row::Action::Unspecified => {
            log::error!("unspecified book action");
            return None;
        },
        finam::proto::marketdata::stream_order_book::row::Action::Remove => "r",
        finam::proto::marketdata::stream_order_book::row::Action::Add => "a",
        finam::proto::marketdata::stream_order_book::row::Action::Update => "u",
    })
}