use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Allows owner_address to set vault code id, to be used
    /// when creating new instances of vaults
    SetVaultCodeId { code_id: u64 },

    /// Allows owner_address to set vault creation fee
    SetVaultCreationFee { amount: Coin },

    /// Creates a new vault by calling the instantiate method of the VAULT_CONTRACT,
    /// which returns a contract address of the new vault.
    MintVault {},

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
    /// Returns Config
    Info {},
}
