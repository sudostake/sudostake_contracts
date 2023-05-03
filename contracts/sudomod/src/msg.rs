use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub staking_denom: String,
    pub owner_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Sets the code_id to be used for creating new instances of vaults
    SetVaultCodeId {},

    /// Creates a new vault by calling the instantiate method of the VAULT_CONTRACT,
    /// which returns a contract address of the new vault
    CreateVault {},

    /// Allows  owner_address to withdraw funds.
    WithdrawBalance {
        to_address: Option<String>,
        funds: Coin,
    },

    /// Allows a vault owner to transfer ownership to another user.
    TransferOwnership { to_address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns InfoResponse
    Info {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {}
