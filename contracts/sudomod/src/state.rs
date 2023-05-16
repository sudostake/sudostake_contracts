use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::Item;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub vault_code_id: Option<u64>,
    pub vault_creation_fee: Option<Coin>
}

// This stores the config variables during initialization of the contract
pub const CONFIG: Item<Config> = Item::new("CONFIG");
