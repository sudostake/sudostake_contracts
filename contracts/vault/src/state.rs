use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin, Timestamp, Uint128};
use cw_storage_plus::Item;

use crate::msg::LiquidityRequestOptionMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: Addr,
    pub acc_manager: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum LiquidityRequestOptionState {
    FixedTermRental {
        requested_amount: Coin,
        start_time: Timestamp,
        last_claim_time: Timestamp,
        end_time: Timestamp,
        can_cast_vote: Option<bool>,
        is_lp_group: Option<bool>,
    },
    FixedInterestRental {
        requested_amount: Coin,
        claimable_tokens: Uint128,
        already_claimed: Uint128,
        can_cast_vote: Option<bool>,
        is_lp_group: Option<bool>,
    },
    FixedTermLoan {
        requested_amount: Coin,
        collateral_amount: Uint128,
        is_lp_group: Option<bool>,
        start_time: Timestamp,
        end_time: Timestamp,
        processing_liquidation: Option<bool>
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ActiveOption {
    pub lender: Option<Addr>,
    pub state: Option<LiquidityRequestOptionState>,
    pub msg: LiquidityRequestOptionMsg,
}

// This stores the config variables during initialization of the contract
pub const CONFIG: Item<Config> = Item::new("CONFIG");

// This stores the state for the active liquidity request option
pub const OPEN_LIQUIDITY_REQUEST: Item<Option<ActiveOption>> = Item::new("OPEN_LIQUIDITY_REQUEST");
