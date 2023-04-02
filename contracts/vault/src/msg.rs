use cosmwasm_std::{Coin, Uint128, VoteOption};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub owner_address: String,
    pub account_manager_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LiquidityRequestOptionMsg {
    FixedTermRental {
        requested_liquidity: Coin,
        duration_in_days: u32,
        is_lp_user: bool,
        can_cast_vote: bool,
    },
    FixedInterestRental {
        requested_liquidity: Coin,
        claimable_tokens: Coin,
        is_lp_user: bool,
        can_cast_vote: bool,
    },
    FixedTermLoan {
        requested_liquidity: Coin,
        to_pay_back: Coin,
        duration_in_days: u32,
        collateral: Coin,
        can_claim_rewards: bool,
        is_lp_user: bool,
        can_cast_vote: bool,
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

    /// Allows the vault owner to withdraw funds from the vault.
    WithdrawBalance {
        to_address: Option<String>,
        funds: Coin,
    },

    /// Allows a vault owner to transfer ownership to another user.
    TransferOwnership {
        to_address: String,
    },

    /// Allows a liquidity provider (which could be an individual or an LP_GROUP contract)
    /// to accept a liquidity request option.
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
    Config {},
}
