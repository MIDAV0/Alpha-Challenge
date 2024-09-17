use anyhow::Result;

use alloy::{
    primitives::{Address, U256},
    providers::RootProvider,
    pubsub::PubSubFrontend,
    sol
};
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use rand::Rng;
use std::sync::Arc;

use crate::utils::constants::*;

sol!(
    #[allow(missing_docs)]
    #[sol(rpc)]
    ERC20,
    "src/data/contract_abis/ERC20.json"
);

pub fn setup_logger() -> Result<()> {
    let colors = ColoredLevelConfig {
        trace: Color::Cyan,
        debug: Color::Magenta,
        info: Color::Green,
        warn: Color::Red,
        error: Color::BrightRed,
        ..ColoredLevelConfig::new()
    };

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "{}[{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                colors.color(record.level()),
                message
            ))
        })
        .chain(std::io::stdout())
        .level(log::LevelFilter::Error)
        .level_for(PROJECT_NAME, LevelFilter::Info)
        .apply()?;

    Ok(())
}

pub fn calculate_next_block_base_fee(
    gas_used: U256,
    gas_limit: U256,
    base_fee_per_gas: U256,
) -> U256 {
    let gas_used = gas_used;

    let mut target_gas_used = gas_limit / U256::from(2u64);
    target_gas_used = if target_gas_used == U256::ZERO {
        U256::from(1u64)
    } else {
        target_gas_used
    };

    let new_base_fee = {
        if gas_used > target_gas_used {
            base_fee_per_gas
                + ((base_fee_per_gas * (gas_used - target_gas_used)) / target_gas_used)
                    / U256::from(8u64)
        } else {
            base_fee_per_gas
                - ((base_fee_per_gas * (target_gas_used - gas_used)) / target_gas_used)
                    / U256::from(8u64)
        }
    };

    let seed = rand::thread_rng().gen_range(0..9);
    new_base_fee + U256::from(seed)
}


pub async fn get_token_balance(
    owner: Address,
    token: Address,
    provider: Arc<RootProvider<PubSubFrontend>>,
) -> Result<U256>
{
    // Initialize ERC20 token contract instance
    let erc20 = ERC20::new(token, provider);    
    // Call the balanceOf function
    let ERC20::balanceOfReturn { balance } = erc20.balanceOf(owner).call().await?;
    
    Ok(balance)
}