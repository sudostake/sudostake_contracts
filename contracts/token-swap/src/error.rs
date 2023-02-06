use cosmwasm_std::{StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Cw20Error(#[from] cw20_base::ContractError),

    #[error("None Error")]
    NoneError {},

    #[error("Max quote token error: max_token: {max_quote_token_amount}, tokens_required: {required_quote_token_amount}")]
    MaxQuoteTokenAmountExceeded {
        max_quote_token_amount: Uint128,
        required_quote_token_amount: Uint128,
    },

    #[error("Insufficient liquidity error: requested: {requested}, available: {available}")]
    InsufficientLiquidityError {
        requested: Uint128,
        available: Uint128,
    },

    #[error("Min base token output error: requested: {requested}, available: {available}")]
    MinBaseTokenOutputError {
        requested: Uint128,
        available: Uint128,
    },

    #[error("Min quote token output error: requested: {requested}, available: {available}")]
    MinQuoteTokenOutputError {
        requested: Uint128,
        available: Uint128,
    },

    #[error("Swap min error: min: {min}, available: {available}")]
    SwapMinError { min: Uint128, available: Uint128 },

    #[error("Swap max error: max: {max}, required: {required}")]
    SwapMaxError { max: Uint128, required: Uint128 },

    #[error("MsgExpirationError")]
    MsgExpirationError {},

    #[error("IncorrectAmountProvided")]
    IncorrectAmountProvided {
        provided: Uint128,
        required: Uint128,
    },

    #[error("Non zero amount for base and quote tokens is expected")]
    NonZeroInputAmountExpected {},

    #[error("No native token provided in pair")]
    NativeTokenNotProvidedInPair {},

    #[error("Base denom is not a native token")]
    InvalidNativeDenom {},

    #[error("Quote denom is not a cw20 token")]
    InvalidQuoteDenom {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
}
