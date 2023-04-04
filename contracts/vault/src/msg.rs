use cosmwasm_std::{Coin, Uint128, VoteOption};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::state::{ActiveOption, Config};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner_address: String,
    pub account_manager_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LiquidityRequestOptionMsg {
    FixedTermRental {
        requested_amount: Coin,
        duration_in_seconds: u64,
        is_lp_group: Option<bool>,
        can_cast_vote: Option<bool>,
    },
    FixedInterestRental {
        requested_amount: Coin,
        claimable_tokens: Uint128,
        is_lp_group: Option<bool>,
        can_cast_vote: Option<bool>,
    },
    FixedTermLoan {
        requested_amount: Coin,
        collateral_amount: Uint128,
        duration_in_seconds: u64,
        can_claim_rewards: Option<bool>,
        is_lp_group: Option<bool>,
        can_cast_vote: Option<bool>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Allows the vault owner to stake the assets to a validator.
    Delegate {
        validator: String,
        amount: Uint128,
    },

    /// Allows the vault owner to redelegate their stake to another validator.
    Redelegate {
        src_validator: String,
        dst_validator: String,
        amount: Uint128,
    },

    /// Allows the vault owner to un-stake the assets from a validator.
    Undelegate {
        validator: String,
        amount: Uint128,
    },

    /// Allows the vault owner to open a liquidity request option
    OpenLRO {
        option: LiquidityRequestOptionMsg,
    },

    /// Allows the vault owner to close a liquidity request option
    /// before the offer is accepted by other market participants.
    ClosePendingLRO {},

    /// Allows a liquidity provider (which could be an individual or 
    /// an LP_GROUP contract to accept a liquidity request option.
    AcceptLRO {
        is_contract_user: Option<bool>,
    },

    // Allows the vault owner(s) to claim delegator rewards
    ClaimDelegatorRewards {},

    /// Allows the vault owner/lender to liquidate collateral
    /// by unstaking the specified amount.
    LiquidateCollateral {},

    /// Allows the vault owner/lender to transfer collateral
    /// to lender's address when funds becomes available
    RepayLoan {},

    /// Allows the vault owner to withdraw funds from the vault.
    WithdrawBalance {
        to_address: Option<String>,
        funds: Coin,
    },

    /// Allows a vault owner to transfer ownership to another user.
    TransferOwnership {
        to_address: String,
    },

    /// Allows vault owner/lender to cast a simple vote
    Vote {
        proposal_id: u64,
        vote: VoteOption,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns InfoResponse
    Info {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {
    pub config: Config,
    pub liquidity_request: Option<ActiveOption>,
}
