use cw20::Denom;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub native_denom: Denom,
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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {}
