use cosmwasm_std::{Addr, Uint128};
use cw20::Denom;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token {
    pub reserve: Uint128,
    pub denom: Denom,
}

#[derive(Debug, PartialEq)]
pub struct TokenAmount {
    pub amount: Uint128,
    pub denom: Denom,
}

#[derive(Debug, PartialEq)]
pub struct SwapPrice {
    pub input: TokenAmount,
    pub output: TokenAmount,
}

pub const LP_TOKEN: Item<Addr> = Item::new("lp_token");
pub const NATIVE_DENOM: Item<Denom> = Item::new("native_denom");
pub const BASE_TOKEN: Item<Token> = Item::new("base_token");
pub const QUOTE_TOKEN: Item<Token> = Item::new("quote_token");
