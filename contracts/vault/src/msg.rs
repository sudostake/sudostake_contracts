use cosmwasm_std::{Coin, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub staking_denom: String,
    pub owner_address: String,
    pub account_manager_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LiquidityRequestOption {
    FixedTermRewardsClaim {
        requested_liquidity: Coin,
        duration_in_days: u32,
    },
    FixedInterestRewardsClaim {
        requested_liquidity: Coin,
        claimable_tokens: Coin,
    },
    FixedTermLoan {
        requested_liquidity: Coin,
        to_pay_back: Coin,
        duration_in_days: u32,
        token_amount_to_liquidate_on_default: Coin,
        can_claim_staking_rewards: bool,
    },
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

    // Allows the vault owner to claim delegator rewards when there is no active LRO
    ClaimDelegatorRewards {
        withdraw: Option<bool>,
    },

    /// Allows the vault owner to open a liquidity request option
    OpenLRO {
        option: LiquidityRequestOption,
    },

    /// Allows the vault owner to close a liquidity request option
    /// before the offer is accepted by other market participants.
    ClosePendingLRO {},

    /// Allows a liquidity provider (which could be an individual or an LP_GROUP contract)
    /// to accept a liquidity request option.
    AcceptLRO {
        is_contract_user: Option<bool>,
    },

    /// Allows the vault owner/controller to process LRO claims.
    ProcessClaimsForLRO {},

    /// Allows the vault owner/controller to
    /// withdraw assets held in the vault based on allowance.
    Withdraw {
        to_address: Option<String>,
        funds: Coin,
    },

    /// Allows a vault owner to transfer ownership to another user.
    Transfer {
        to_address: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns InfoResponse
    Info {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {}
