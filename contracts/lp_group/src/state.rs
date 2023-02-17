use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub staking_denom: String,
    pub owner: Addr,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MemberData {
    pub address: Addr,
}

// This stores the config variables during initialization of the contract
pub const CONFIG: Item<Config> = Item::new("CONFIG");

// This store the list of group members
pub const MEMBERS: Map<&str, MemberData> = Map::new("MEMBERS");
