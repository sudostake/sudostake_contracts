use cosmwasm_std::{Coin, StdError, Uint128};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("InvalidInputAmount:  required: {required:?}, received: {received:?}")]
    InvalidInputAmount {
        required: Uint128,
        received: Uint128,
    },

    #[error("ValidatorIsInactive: {validator:?}")]
    ValidatorIsInactive { validator: String },

    #[error("LenderCannotRedelegateFromActiveValidator: {validator:?}")]
    LenderCannotRedelegateFromActiveValidator { validator: String },

    #[error("LiquidityRequestIsActive")]
    LiquidityRequestIsActive {},

    #[error("InvalidLiquidityRequestOption")]
    InvalidLiquidityRequestOption {},

    #[error("MaxUndelegateAmountExceeded: amount: {amount:?}, validator_delegation: {validator_delegation:?}")]
    MaxUndelegateAmountExceeded {
        amount: Uint128,
        validator_delegation: Uint128,
    },

    #[error("InsufficientBalance: Required {required:?}, Available {available:?}")]
    InsufficientBalance { required: Coin, available: Coin },

    #[error("Repay: {amount:?}, owed to the lender for the defaulted fixed term loan")]
    ClearOutstandingDebt { amount: Coin },

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
