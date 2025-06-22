use std::sync::{Arc, Mutex};

use tonic::{metadata::{Ascii, MetadataValue}, service::Interceptor, Request};



#[derive(Debug, Clone)]
pub struct TokenInterceptor {
    token: Arc<Mutex<String>>,
}

impl TokenInterceptor {
    pub async fn new(token: impl ToString) -> Self {
        Self { token: Arc::new(Mutex::new(token.to_string())) }
    }
}

impl Interceptor for TokenInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, tonic::Status> {
        let x: MetadataValue<Ascii> = self.token.lock().unwrap().parse().unwrap();
        req.metadata_mut().append(
            "authorization",
            x.clone(),
        );
        Ok(req)
    }
}
