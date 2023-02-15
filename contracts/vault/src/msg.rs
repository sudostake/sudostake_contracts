use cosmwasm_std::Uint128;
use cw20::Denom;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub native_denom: Denom,
}

/**
 * LRO: Liquidity request option
 */
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Allows the vault owner to stake the assets to a validator.
    Delegate {
        validator: String,
        amount: Uint128,
    },

    /// Allows the vault owner to un-stake the assets from a validator.
    Undelegate {
        validator: String,
        amount: Uint128,
    },

    /// Allows the vault owner to redelegate their stake to another validator.
    Redelegate {
        src_validator: String,
        dst_validator: String,
        amount: Uint128,
    },

    /// Allows the vault owner to open a liquidity request option
    OpenLRO {
        // todo
    },

    /// Allows the vault owner to close a liquidity request option
    /// before the offer is accepted by other market participants.
    CloseLRO {
        // todo
    },

    /// Allows a liquidity provider (which could be an individual or an LP_GROUP)
    /// to accept a liquidity request option.
    AcceptLRO {
        // todo
    },

    /// Allows the vault owner/controller to process LRO claims.
    ProcessLROClaims {
        // todo
    },

    // Allows the vault owner to claim delegator rewards when there is no active LRO
    ClaimDelegatorRewards {
        withdraw: Option<bool>,
    },

    /// Allows the vault owner/controller to
    /// withdraw assets held in the vault based on allowance.
    WithdrawFunds {
        // todo
    },

    /// Allows a vault owner to transfer ownership to another user.
    Transfer {
        // todo
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {}
