use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
use cw_storage_plus::Item;
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

// contract info
pub const CONTRACT_NAME: &str = "vault_contract";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// Only INSTANTIATOR_ADDR can call the InstantiateMsg
pub const INSTANTIATOR_ADDR: &str = "contract1";

// Minimum duration between calls to unbond collateral during liquidation
pub const STAKE_LIQUIDATION_INTERVAL: u64 = 60 * 60 * 24 * 30;

// This stores the config variables during initialization of the contract
pub const CONFIG: Item<Config> = Item::new("CONFIG");

// This stores the state for the active liquidity request option
pub const OPEN_LIQUIDITY_REQUEST: Item<Option<ActiveOption>> = Item::new("OPEN_LIQUIDITY_REQUEST");
