use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Timestamp};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub last_vault_info_update: Option<Timestamp>,
    pub vault_creation_fee: Option<Coin>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultCodeInfo {
    pub id: u64,
    pub code_id: u64,
}

// This stores the config variables during initialization of the contract
pub const CONFIG: Item<Config> = Item::new("CONFIG");

// This keeps track of the number of items in the vault list
pub const ENTRY_SEQ: Item<u64> = Item::new("ENTRY_SEQ");

// This keeps track of a mapping between the ENTRY_SEQ : VaultCodeInfo
pub const VAULT_CODE_LIST: Map<u64, VaultCodeInfo> = Map::new("VAULT_CODE_LIST");

// Limits for the custom range query
pub const MAX_LIMIT: u32 = 30;
pub const DEFAULT_LIMIT: u32 = 12;
