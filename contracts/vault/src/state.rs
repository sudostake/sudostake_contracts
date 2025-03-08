use crate::types::{ActiveOption, Config};
use cw_storage_plus::Item;

// contract info
pub const CONTRACT_NAME: &str = "vault_contract";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Only INSTANTIATOR_ADDR can call the InstantiateMsg
pub const INSTANTIATOR_ADDR: &str = "contract1";

// Minimum duration between calls to unbond collateral during liquidation
pub const STAKE_LIQUIDATION_INTERVAL: u64 = 60 * 60 * 24 * 30;

// This stores the config variables during initialization of the contract
pub const CONFIG: Item<Config> = Item::new("CONFIG");

// This stores the state for the active liquidity request option
pub const OPEN_LIQUIDITY_REQUEST: Item<Option<ActiveOption>> = Item::new("OPEN_LIQUIDITY_REQUEST");
