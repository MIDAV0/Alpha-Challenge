use std::sync::Arc;
use alloy::{
    dyn_abi::DynSolValue,
    primitives::{Address, FixedBytes, U256},
    providers::{Provider, RootProvider},
    pubsub::PubSubFrontend,
};
use alloy_contract::Interface;
use std::collections::HashSet;
// use alloy::rpc::types::trace::geth::{CallFrame, CallConfig, GethDebugTracingCallOptions, GethDebugBuiltInTracerType, GethDebugTracerType, GethDebugTracerConfig, GethTrace};

use eyre::Result;
use super::helpers::NewPendingTx;
use crate::utils::constants::VAULT_DEPLOY_BYTECODE;

pub static VAULT_DEPOSIT_EVENT_ID: &str = "0x2c32e4d4"; // Deposit(address,address,uint256,uint256)

pub struct VaultDepositInfo {
    pub tx_hash: FixedBytes<32>,
    pub vault_address: Address,
    pub amount: U256,
}

// pub async fn debug_trace_call(
//     provider: &Arc<RootProvider<PubSubFrontend>>,
//     new_block: &NewBlock,
//     pending_tx: &NewPendingTx,
// ) -> Result<Option<CallFrame>> {
//     let mut opts: GethDebugTracingCallOptions = GethDebugTracingCallOptions::default();
//     let mut call_config = CallConfig::default();
//     call_config.with_log = Some(true);

//     opts.tracing_options.tracer = Some(GethDebugTracerType::BuiltInTracer(
//         GethDebugBuiltInTracerType::CallTracer,
//     ));
//     opts.tracing_options.tracer_config = GethDebugTracerConfig(serde_json::to_value(call_config)?);

//     let block_number = new_block.block_number;
//     let mut tx = pending_tx.tx.clone();
//     let nonce = provider
//         .get_transaction_count(tx.from)
//         .await
//         .unwrap_or_default();
//     tx.nonce = nonce;
    
//     let trace = provider
//         .debug_trace_call(&tx, Some(block_number.into()), opts)
//         .await;

//     match trace {
//         Ok(trace) => match trace {
//             GethTrace::CallTracer(frame) => Ok(Some(frame)),
//             _ => Ok(None),
//         },
//         _ => Ok(None),
//     }
//     Ok(None)
// }

// pub fn extract_logs(call_frame: &CallFrame, logs: &mut Vec<CallLogFrame>) {
//     logs.extend(call_frame.logs.iter().cloned());

//     for call in &call_frame.calls {
//         extract_logs(call, logs);
//     }
// }

pub async fn get_deployment_contract_address(
    provider: &Arc<RootProvider<PubSubFrontend>>,
    deployer: Address,
) -> Result<Address> {
    let nonce = provider.get_transaction_count(deployer).await?;
    let address = deployer.create(nonce);
    Ok(address)
}

pub async fn extract_vault_creation_info(
    provider: &Arc<RootProvider<PubSubFrontend>>,
    pending_tx: &NewPendingTx,
) -> Result<Option<Address>> {
    if !pending_tx.tx.to.is_none() || pending_tx.tx.to != Some(Address::ZERO) {
        return Ok(None);
    }

    if pending_tx.tx.input != VAULT_DEPLOY_BYTECODE {
        return Ok(None);
    }

    let deploy_address = get_deployment_contract_address(provider, pending_tx.tx.from).await?;

    Ok(Some(deploy_address))
}

pub async fn extract_vault_deposit_info(
    pending_tx: &NewPendingTx,
    vault_interface: &Interface,
    vaults_set: &mut HashSet<Address>,
) -> Result<Option<VaultDepositInfo>> {
    let tx_hash = pending_tx.tx.hash;

    if let Some(to_address) = pending_tx.tx.to {
        if vaults_set.contains(&to_address) {
            let slector = &format!("{:?}", pending_tx.tx.input)[0..10];
            if slector == VAULT_DEPOSIT_EVENT_ID {
                let vault_address = to_address;
                let input_slice: &[u8] = &pending_tx.tx.input[..];
                let decoded_input = vault_interface.decode_input("deposit", input_slice, true)?;
                let amount = match decoded_input[0] {
                    DynSolValue::Uint(a,_) => a,
                    _ => return Ok(None),
                };
                vaults_set.remove(&vault_address);
                return Ok(Some(VaultDepositInfo {
                    tx_hash,
                    vault_address,
                    amount,
                }));
            }
        }
    }

    Ok(None)
}
