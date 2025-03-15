use crate::types::{ActiveOption, Config};
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

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
pub const LIQUIDITY_REQUEST_STATE: Item<Option<ActiveOption>> =
    Item::new("LIQUIDITY_REQUEST_STATE");

// This stores the list for all counter offers
// https://docs.rs/cw-storage-plus/1.0.1/cw_storage_plus/struct.Map.html
// https://cosmwasm.cosmos.network/cw-storage-plus/containers/map
pub const MAX_COUNTER_OFFERS: usize = 10;
pub const COUNTER_OFFER_LIST: Map<Addr, Uint128> = Map::new("COUNTER_OFFER_LIST");
