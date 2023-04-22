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
    /// Creates a new vault by calling the instantiate method of the VAULT_CONTRACT,
    /// which returns a contract address, that is then associated with the `msg.sender`
    CreateVault {},

    /// Creates a new LP_GROUP by calling the instantiate method of the LP_GROUP_CONTRACT,
    /// which returns a contract address, that is then associated with the `msg.sender`.
    CreateLPGroup {},

    /// Withdraw generated fees to the address provided by the owner
    WithdrawFees { to_address: String },

    /// Allows the current owner to transfer ownership to another user.
    Transfer { to_address: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns InfoResponse
    Info {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {}
