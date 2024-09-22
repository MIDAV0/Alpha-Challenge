use std::{collections::HashSet, sync::Arc};
use alloy_contract::Interface;
use tokio::sync::broadcast::Sender;
use alloy::{
    providers::RootProvider,
    pubsub::PubSubFrontend,
    primitives::Address,
};
use crate::utils::helpers::Event;
use crate::utils::simulation::{extract_vault_creation_info, extract_vault_deposit_info};


pub async fn run_strategy(provider: Arc<RootProvider<PubSubFrontend>>, event_sender: Sender<Event>) {
    let mut event_receiver = event_sender.subscribe();
    let mut vault_set: HashSet<Address> = HashSet::new();

    loop {
        match event_receiver.recv().await {
            Ok(event) => match event {
                Event::PendingTx(mut pending_tx) => {
                    let vault_address: Option<Address> = match extract_vault_creation_info(&provider, &mut pending_tx).await {
                        Ok(vault_info) => vault_info,
                        Err(_) => None,
                    };
                    if vault_address.is_none() {
                        continue;
                    }
                    // Back run vault deposit



                    vault_set.insert(vault_address.unwrap().vault_address);

                    let deposit_info = match extract_vault_deposit_info(
                        &pending_tx,
                        &vault_interface,
                        &mut vault_set,
                    ).await {
                        Ok(deposit_info) => deposit_info,
                        Err(_) => None,
                    };

                    if deposit_info.is_none() {
                        continue;
                    }

                    // Front run user deposit

                },
                _ => {}
            },
            _ => {}
        }
    }
}