use cosmwasm_std::{Coin, StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("InsufficientBalance")]
    InsufficientBalance { required: Coin, available: Coin },

    #[error("ValidatorIsInactive")]
    ValidatorIsInactive { validator: String },

    #[error("InvalidLiquidityRequestOption")]
    InvalidLiquidityRequestOption {},

    #[error("MaxUndelegateAmountExceeded: amount: {amount:?}, validator_delegation: {validator_delegation:?}")]
    MaxUndelegateAmountExceeded {
        amount: Uint128,
        validator_delegation: Uint128,
    },

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
