use std::{sync::{Arc, Mutex, RwLock}, time::Duration};

use tonic::{metadata::{Ascii, MetadataValue}, service::Interceptor, transport::Channel, Request};

use crate::proto::auth::{auth_service_client::AuthServiceClient, AuthRequest};

pub const TOKEN_UPDATE_PERIOD: Duration = Duration::from_secs(14*60); //каждые 14 минут

pub type Token = MetadataValue<Ascii>;

#[derive(Debug, Clone)]
pub struct TokenInterceptor {
    token: Arc<Mutex<Token>>,
}

impl TokenInterceptor {
    pub async fn with_duration(channel: Channel, secret: impl ToString, update_period: Duration) -> Result<Self, tonic::Status> {
        let secret = secret.to_string();
        let token = Arc::new(Mutex::new(get_token(channel.clone(), secret.clone()).await?));
        log::info!("JWT received");
        let weak = Arc::downgrade(&token);
        tokio::spawn(async move {
            tokio::time::sleep(update_period).await;
            while let Some(token) = weak.upgrade() {
                match get_token(channel.clone(), secret.clone()).await {
                    Ok(updated) => {
                        *token.lock().unwrap() = updated;
                        drop(token);
                        log::info!("JWT updated");
                        tokio::time::sleep(update_period).await;
                    },
                    Err(e) => {
                        log::error!("Cannot update token: {e}");
                        tokio::time::sleep(crate::RETRY_DELAY).await;
                    }
                }
            }
            log::info!("token updating stopped");
        });
        Ok(Self { token })
    }
    pub async fn new(channel: Channel, secret: impl ToString) -> Result<Self, tonic::Status> {
        Self::with_duration(channel, secret, TOKEN_UPDATE_PERIOD).await
    }
    pub fn get_token(&self) -> Arc<Mutex<Token>> {
        self.token.clone()
    }
}

impl Interceptor for TokenInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, tonic::Status> {
        let token = self.token.lock().unwrap().clone();
        req.metadata_mut().append(
            "authorization",
            token,
        );
        Ok(req)
    }
}

async fn get_token(channel: Channel, secret: String) -> Result<MetadataValue<Ascii>, tonic::Status> {
    let mut serv = AuthServiceClient::new(channel);
    let crate::proto::auth::AuthResponse {token} = serv.auth(AuthRequest { secret }).await?.into_inner();
    Ok(token.parse().unwrap())
}