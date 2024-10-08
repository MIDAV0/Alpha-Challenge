use eyre::Result;
use alloy::providers::{ProviderBuilder, WsConnect};
use log::info;
use std::sync::Arc;
use tokio::sync::broadcast::{self, Sender};
use tokio::task::JoinSet;

use vault::utils::constants::Env;
use vault::utils::utils::setup_logger;
use vault::utils::helpers::{stream_pending_transactions, Event};
use vault::utils::strategy::run_strategy;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();
    setup_logger().unwrap();

    info!("Starting Vault Exploit");

    let env = Env::new();

    let ws = WsConnect::new(env.wss_url.clone());
    let provider = Arc::new(ProviderBuilder::new().with_recommended_fillers().on_ws(ws).await?);

    let (event_sender, _): (Sender<Event>, _) = broadcast::channel(512);

    let mut set = JoinSet::new();

    set.spawn(stream_pending_transactions(
        provider.clone(),
        event_sender.clone(),
    ));

    set.spawn(run_strategy(
        provider.clone(),
        event_sender.clone()
    ));

    while let Some(res) = set.join_next().await {
        info!("{:?}", res);
    }

    Ok(())
}
