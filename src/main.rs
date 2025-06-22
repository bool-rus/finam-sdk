use std::{env, error::Error};

use tonic::{service::{interceptor::InterceptedService, Interceptor}, transport::{Channel, ClientTlsConfig}, Request};

use crate::{auth::TokenInterceptor, proto::{accounts::{accounts_service_client::AccountsServiceClient, GetAccountRequest}, auth::{auth_service_client::AuthServiceClient, AuthResponse, TokenDetailsRequest, TokenDetailsResponse}, google::r#type::Interval}};
mod proto;
mod auth;


#[tokio::main]
async fn main() -> Result<(),Box<dyn Error>> {
    use proto::grpc::tradeapi::v1::auth::AuthRequest;
    let secret = env::var("TOKEN").expect("Need token");
    let tls = ClientTlsConfig::new().with_native_roots();
    let channel = Channel::from_static("https://api.finam.ru").tls_config(tls)?.connect_lazy();
    let mut auth = AuthServiceClient::new(channel.clone());
    let AuthResponse { token } = auth.auth(AuthRequest{ secret }).await?.into_inner();
    let details = auth.token_details(TokenDetailsRequest{token: token.clone()}).await?.into_inner();
    let account_id = details.account_ids.first().unwrap().clone();
    println!("{details:#?}");
    let channel = InterceptedService::new(channel, TokenInterceptor::new(token).await);
    let mut accs = AccountsServiceClient::new(channel.clone());
    let a = accs.get_account(GetAccountRequest{ account_id }).await?.into_inner();
    println!("{a:#?}");
    //println!("Hello, world! token: {token}");
    Ok(())
}
