use cosmwasm_std::{StdError, Coin};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("InsufficientBalance: Required {required:?}, Available {available:?}")]
    InsufficientBalance { required: Coin, available: Coin },

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Minimum vault code update interval not reached")]
    MinVaultCodeUpdateIntervalNotReached {},

    #[error("Custom Error val: {val:?}")]
    CustomError { val: String },
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
}
