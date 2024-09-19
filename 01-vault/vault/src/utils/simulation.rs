use std::sync::Arc;
use alloy::{
    primitives::{Address, U128, U256, U64}, providers::{Provider, RootProvider}, pubsub::PubSubFrontend, rpc::types::trace::geth::CallLogFrame
};
use alloy::rpc::types::trace::geth::{CallFrame, CallConfig, GethDebugTracingCallOptions, GethDebugBuiltInTracerType, GethDebugTracerType, GethDebugTracerConfig, GethTrace};
use eyre::Result;
use super::helpers::{NewBlock, NewPendingTx};

pub static VAULT_ADDRESS: &str = "0x6E1241BAAc5F7cb671Df30f7243BaC1987aDd7E1";
pub static VAULT_DEPOSIT_ID: &str = "0xd78ad95f";

pub async fn debug_trace_call(
    provider: &Arc<RootProvider<PubSubFrontend>>,
    new_block: &NewBlock,
    pending_tx: &NewPendingTx,
) -> Result<Option<CallFrame>> {
    let mut opts: GethDebugTracingCallOptions = GethDebugTracingCallOptions::default();
    let mut call_config = CallConfig::default();
    call_config.with_log = Some(true);

    opts.tracing_options.tracer = Some(GethDebugTracerType::BuiltInTracer(
        GethDebugBuiltInTracerType::CallTracer,
    ));
    opts.tracing_options.tracer_config = GethDebugTracerConfig(serde_json::to_value(call_config)?);

    let block_number = new_block.block_number;
    let mut tx = pending_tx.tx.clone();
    let nonce = provider
        .get_transaction_count(tx.from)
        .await
        .unwrap_or_default();
    tx.nonce = nonce;
    
    let trace = provider
        .debug_trace_call(&tx, Some(block_number.into()), opts)
        .await;

    match trace {
        Ok(trace) => match trace {
            GethTrace::CallTracer(frame) => Ok(Some(frame)),
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

pub fn extract_logs(call_frame: &CallFrame, logs: &mut Vec<CallLogFrame>) {
    logs.extend(call_frame.logs.iter().cloned());

    for call in &call_frame.calls {
        extract_logs(call, logs);
    }
}

pub async fn extract_vault_interaction(
    provider: &Arc<RootProvider<PubSubFrontend>>,
    new_block: &NewBlock,
    pending_tx: &NewPendingTx,
    target_vault_address: Address,
) -> Result<()> {
    let tx_hash = pending_tx.tx.hash;

    let frame = debug_trace_call(provider, new_block, pending_tx).await?;
    let frame = frame.unwrap();

    let mut logs = Vec::new();
    extract_logs(&frame, &mut logs);

    for log in &logs {
        if let Some(contract_address) = log.address {
            if contract_address.to_string() != target_vault_address.to_string() {continue;}
        } else {
            continue;
        }
        match &log.topics {
            Some(topics) => {
                if topics.len() > 1 {
                    let selector = &format!("{:?}", topics[0])[0..10];
                    let is_vault_deposit = selector == VAULT_DEPOSIT_ID;

                    if is_vault_deposit {

                    }
                }
            }
            _ => {},
        }
    }


    Ok(())
}