use crate::state::VaultCodeInfo;
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

    /// Allows owner_address to set an amount to be paid by info.sender
    /// when calling MintVault.
    SetVaultCreationFee { amount: Coin },

    /// Ensures that all instances of vaults created from code_id
    /// is paid for, as only the instance of sudomod contract set as INSTANTIATOR_ADDR in the
    /// vault contract's source code can call the instantiate method of the VAULT_CONTRACT
    MintVault {},

    /// Allows owner_address to withdraw funds from the contract account.
    WithdrawBalance {
        to_address: Option<String>,
        funds: Coin,
    },

    /// Allows owner_address to transfer ownership to another owner's address
    /// Note: To burn this contract account, set to_address = env.contract.address
    TransferOwnership { to_address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns Config
    Info {},

    /// Returns VaultCodeListResponse
    QueryVaultCodeList {
        start_after: Option<u64>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultCodeListResponse {
    pub entries: Vec<VaultCodeInfo>,
}
