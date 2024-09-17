
use log::info;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::broadcast::Sender;
use alloy::{
    providers::RootProvider,
    pubsub::PubSubFrontend,
};

use crate::utils::helpers::Event;

pub async fn run_sandwich_strategy(provider: Arc<RootProvider<PubSubFrontend>>, event_sender: Sender<Event>) {
    let mut event_receiver = event_sender.subscribe();

    loop {
        match event_receiver.recv().await {
            Ok(event) => match event {
                Event::PendingTx(mut pending_tx) => {
                    info!("{:?}", pending_tx);
                },
                _ => {}
            },
            _ => {}
        }
    }
}