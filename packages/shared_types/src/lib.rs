use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


// TODO complete the description of this enum
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VaultEvents {
    ClaimRewards,
    BeginLiquidation,
    FinalizedClaim,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProcessPoolHook {
    pub vault_id: u16,
    pub event: VaultEvents,
}
