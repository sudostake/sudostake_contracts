use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub from_code_id: u64,
    pub index_number: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ActiveOption {
    pub lender: Option<Addr>,
    pub msg: LiquidityRequestMsg,
    pub state: Option<LiquidityRequestState>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LiquidityRequestMsg {
    FixedTermRental {
        requested_amount: Coin,
        duration_in_seconds: u64,
        can_cast_vote: bool,
    },
    FixedInterestRental {
        requested_amount: Coin,
        claimable_tokens: Uint128,
        can_cast_vote: bool,
    },
    FixedTermLoan {
        requested_amount: Coin,
        /// Implicitly denominated in requested_amount.denom
        interest_amount: Uint128,
        /// Implicitly denominated in bonded_denom
        collateral_amount: Uint128,
        duration_in_seconds: u64,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CounterOfferOperator {
    Add,
    Sub,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum LiquidityRequestState {
    FixedTermRental {
        requested_amount: Coin,
        start_time: Timestamp,
        last_claim_time: Timestamp,
        end_time: Timestamp,
        can_cast_vote: bool,
    },
    FixedInterestRental {
        requested_amount: Coin,
        claimable_tokens: Uint128,
        already_claimed: Uint128,
        can_cast_vote: bool,
    },
    FixedTermLoan {
        requested_amount: Coin,
        collateral_amount: Uint128,
        interest_amount: Uint128,
        start_time: Timestamp,
        end_time: Timestamp,
        processing_liquidation: bool,
        already_claimed: Uint128,
        last_liquidation_date: Option<Timestamp>,
    },
}

// True  = vault has an open liquidity request
// False = vault does not have an open liquidity request
type WithRequestLiquidity = bool;

#[derive(PartialEq)]
pub enum ActionTypes {
    Delegate,
    Redelegate,
    Undelegate(WithRequestLiquidity),
    RequestLiquidity(WithRequestLiquidity),
    ClosePendingLiquidityRequest(WithRequestLiquidity),
    AcceptLiquidityRequest,
    ClaimDelegatorRewards,
    LiquidateCollateral(WithRequestLiquidity),
    RepayLoan(WithRequestLiquidity),
    WithdrawBalance,
    TransferOwnership,
    Vote,
}
