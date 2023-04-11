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
        can_cast_vote: Option<bool>,
    },
    FixedInterestRental {
        requested_amount: Coin,
        claimable_tokens: Uint128,
        can_cast_vote: Option<bool>,
    },
    FixedTermLoan {
        requested_amount: Coin,
        /// Implicitly denominated in requested_amount.denom
        interest_amount: Uint128,
        /// Implicitly denominated in staked tokens
        collateral_amount: Uint128,
        duration_in_seconds: u64,
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
    OpenLiquidityRequest {
        option: LiquidityRequestOptionMsg,
    },

    /// Allows the vault owner to close a liquidity request option
    /// before the offer is accepted by other market participants.
    CloseLiquidityRequest {},

    /// Allows a liquidity provider (which could be an individual or
    /// an LP_GROUP contract to accept a liquidity request option.
    AcceptLiquidityRequest {
        is_lp_group: Option<bool>,
    },

    // Allows the vault owner(s) to claim delegator rewards
    ClaimDelegatorRewards {},

    /// Allows the vault owner to repay the amount borrowed from the lender
    /// before the expiry date stated in the option
    RepayLoan {},

    /// Allows the vault owner/lender to liquidate collateral
    /// by unstaking the specified amount owed to the lender.
    LiquidateCollateral {},

    /// Allows the vault owner/lender to withdraw funds from the vault.
    /// While liquidation is processing, the lender's withdrawal
    /// is prioritized over the vault owner.
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
