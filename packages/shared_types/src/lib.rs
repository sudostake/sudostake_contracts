use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// TODO complete the description of this enum
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VaultEvents {
    ClaimedRewards,
    CollateralLiquidationStarted,
    FinalizedClaim,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ProcessPoolHook {
    pub vault_address: String,
    pub event: VaultEvents,
}
