use cosmwasm_std::{Addr, Uint128};
use cw20::{Denom, Expiration};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Note: This contract supports
// Native : Native
// Native : IBC
// Native : Cw20
// Token pairings
// Also the contract is currently constrained to the use of only the protocol staking token as base
// while any other token can be used as quote
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub native_denom: Denom,
    pub base_denom: Denom,
    pub quote_denom: Denom,
    pub lp_token_code_id: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum TokenSelect {
    Base,
    Quote,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    AddLiquidity {
        base_token_amount: Uint128,
        max_quote_token_amount: Uint128,
        expiration: Option<Expiration>,
    },

    RemoveLiquidity {
        amount: Uint128,
        min_base_token_output: Uint128,
        min_quote_token_output: Uint128,
        expiration: Option<Expiration>,
    },

    Swap {
        input_token: TokenSelect,
        input_amount: Uint128,
        output_amount: Uint128,
        expiration: Option<Expiration>,
    },

    SwapAndSendTo {
        input_token: TokenSelect,
        input_amount: Uint128,
        output_amount: Uint128,
        recipient: Addr,
        expiration: Option<Expiration>,
    },

    // Chained swap converting Q -> B and B -> Q' by leveraging two swap contracts
    PassThroughSwap {
        quote_input_amount: Uint128,
        output_amm_address: Addr,
        min_quote_output_amount: Uint128,
        expiration: Option<Expiration>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Returns information about the current state of the pool
    Info {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InfoResponse {
    pub base_reserve: Uint128,
    pub base_denom: Denom,
    pub quote_reserve: Uint128,
    pub quote_denom: Denom,
    pub lp_token_supply: Uint128,
    pub lp_token_address: Addr,
}
