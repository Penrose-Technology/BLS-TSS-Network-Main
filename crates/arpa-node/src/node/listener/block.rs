use super::Listener;
use crate::node::{
    error::NodeResult,
    event::new_block::NewBlock,
    queue::{event_queue::EventQueue, EventPublisher},
};
use arpa_node_contract_client::provider::{BlockFetcher, ChainProviderBuilder};
use arpa_node_core::ChainIdentity;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct BlockListener<I: ChainIdentity + ChainProviderBuilder> {
    chain_id: usize,
    chain_identity: Arc<RwLock<I>>,
    eq: Arc<RwLock<EventQueue>>,
}

impl<I: ChainIdentity + ChainProviderBuilder> BlockListener<I> {
    pub fn new(
        chain_id: usize,
        chain_identity: Arc<RwLock<I>>,
        eq: Arc<RwLock<EventQueue>>,
    ) -> Self {
        BlockListener {
            chain_id,
            chain_identity,
            eq,
        }
    }
}

#[async_trait]
impl<I: ChainIdentity + ChainProviderBuilder + Sync + Send> EventPublisher<NewBlock>
    for BlockListener<I>
{
    async fn publish(&self, event: NewBlock) {
        self.eq.read().await.publish(event).await;
    }
}

#[async_trait]
impl<I: ChainIdentity + ChainProviderBuilder + Sync + Send + 'static> Listener
    for BlockListener<I>
{
    async fn listen(&self) -> NodeResult<()> {
        let client = self.chain_identity.read().await.build_chain_provider();
        let chain_id = self.chain_id;
        let eq = self.eq.clone();

        client
            .subscribe_new_block_height(move |block_height: usize| {
                let eq = eq.clone();
                async move {
                    eq.read()
                        .await
                        .publish(NewBlock {
                            chain_id,
                            block_height,
                        })
                        .await;

                    Ok(())
                }
            })
            .await?;

        Ok(())
    }
}
