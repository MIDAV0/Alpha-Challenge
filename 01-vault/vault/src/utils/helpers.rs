use alloy::{
    primitives::{U128, U256, U64}, providers::{Provider, RootProvider}, pubsub::PubSubFrontend, rpc::types::Transaction
};
use std::sync::Arc;
use tokio::sync::broadcast::Sender;
use tokio_stream::StreamExt;

use crate::utils::utils::calculate_next_block_base_fee;

#[derive(Default, Debug, Clone)]
pub struct NewBlock {
    pub block_number: U64,
    pub base_fee: U128,
    pub next_base_fee: U256,
}

#[derive(Debug, Clone)]
pub struct NewPendingTx {
    pub added_block: Option<U64>,
    pub tx: Transaction,
}

impl Default for NewPendingTx {
    fn default() -> Self {
        Self {
            added_block: None,
            tx: Transaction::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Block(NewBlock),
    PendingTx(NewPendingTx),
}

pub async fn stream_new_blocks(provider: Arc<RootProvider<PubSubFrontend>>, event_sender: Sender<Event>) {
    let sub = provider.subscribe_blocks().await.unwrap();
    let mut stream = sub.into_stream().filter_map(|block| match block.header.number {
        Some(number) => Some(NewBlock {
            block_number: U64::from(number),
            base_fee: U128::from(block.header.base_fee_per_gas.unwrap_or_default()),
            next_base_fee: U256::from(calculate_next_block_base_fee(
                U256::from(block.header.gas_used),
                U256::from(block.header.gas_limit),
                U256::from(block.header.base_fee_per_gas.unwrap_or_default()),
            )),
        }),
        None => None,
    });

    while let Some(block) = stream.next().await {
        match event_sender.send(Event::Block(block)) {
            Ok(_) => {}
            Err(_) => {}
        }
    }
}

pub async fn stream_pending_transactions(provider: Arc<RootProvider<PubSubFrontend>>, event_sender: Sender<Event>) {
    let sub = provider.subscribe_pending_transactions().await.unwrap();
    let mut stream = sub.into_stream().take(256).fuse();

    while let Some(tx_hash) = stream.next().await {
        match provider.get_transaction_by_hash(tx_hash).await {
            Ok(tx) => {
                match event_sender.send(Event::PendingTx(NewPendingTx {added_block: None, tx: tx.unwrap()} )) {
                        Ok(_) => {}
                        Err(_) => {}
                }                
            }
            Err(_) => {}
        }
    };
}