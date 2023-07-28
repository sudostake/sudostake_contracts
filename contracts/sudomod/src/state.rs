use cosmwasm_std::{Addr, Coin, Timestamp};
use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub vault_code_info_updated_at: Option<Timestamp>,
    pub vault_creation_fee: Option<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultCodeInfo {
    pub id: u64,
    pub code_id: u64,
}

// contract info
pub const CONTRACT_NAME: &str = "sudomod";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Stores the config variables during initialization of the contract
pub const CONFIG: Item<Config> = Item::new("CONFIG");

// Keeps track of a mapping between the VAULT_CODE_SEQ : VaultCodeInfo
pub const VAULT_CODE_LIST: Map<u64, VaultCodeInfo> = Map::new("VAULT_CODE_LIST");

// Keeps track of the number of items in VAULT_CODE_LIST
pub const VAULT_CODE_SEQ: Item<u64> = Item::new("VAULT_CODE_SEQ");

// Keeps count of vaults instantiated by this contract
pub const VAULT_INSTANTIATION_SEQ: Item<u64> = Item::new("VAULT_INSTANTIATION_SEQ");

// This is the minimum duration in seconds between calls to SetVaultCodeId
pub const MIN_VAULT_CODE_UPDATE_INTERVAL: u64 = 60 * 60 * 24 * 30;

// Limits for the custom range query for VAULT_CODE_LIST
pub const MAX_LIMIT: u32 = 36;
pub const DEFAULT_LIMIT: u32 = 12;
