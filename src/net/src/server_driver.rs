use crate::Client;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;

#[async_trait]
pub trait ServerDriver
where
    Self: 'static + Send + Sync,
{
    async fn on_client_in(&self, client: Arc<Client>, addr: SocketAddr);
    async fn on_client_out(&self, client_id: usize);
}
