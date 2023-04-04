use crate::{
    state::{OPEN_LIQUIDITY_REQUEST, CONFIG},
    ContractError,
};
use cosmwasm_std::{Addr, DepsMut};

// The optional bool Indicates if there is an open liquidity request option on the vault
// which may have already been funded or pending funding
#[derive(PartialEq)]
pub enum ActionTypes {
    Delegate(bool),
    Redelegate,
    Undelegate(bool),
    OpenLRO(bool),
    ClosePendingLRO(bool),
    WithdrawBalance,
    TransferOwnership,
    AcceptLRO,
    ClaimDelegatorRewards,
    LiquidateCollateral,
    RepayLoan,
    Vote(bool),
}

const OWNER_AUTHORIZATIONS: [ActionTypes; 11] = [
    ActionTypes::Delegate(false),
    ActionTypes::Redelegate,
    ActionTypes::Undelegate(false),
    ActionTypes::OpenLRO(false),
    ActionTypes::ClosePendingLRO(true),
    ActionTypes::WithdrawBalance,
    ActionTypes::TransferOwnership,
    ActionTypes::ClaimDelegatorRewards,
    ActionTypes::LiquidateCollateral,
    ActionTypes::RepayLoan,
    ActionTypes::Vote(false),
];

const LENDER_AUTHORIZATIONS: [ActionTypes; 4] = [
    ActionTypes::ClaimDelegatorRewards,
    ActionTypes::LiquidateCollateral,
    ActionTypes::RepayLoan,
    ActionTypes::Vote(true),
];

// There is an activeliquidity requestthat does not yet have a lender
const OPEN_AUTHORIZATIONS: [ActionTypes; 1] = [ActionTypes::AcceptLRO];

pub fn authorize(
    deps: &DepsMut,
    caller: Addr,
    action_type: ActionTypes,
) -> Result<(), ContractError> {
    // Check if the caller has owner authorizations on the vault
    let config = CONFIG.load(deps.storage)?;
    if caller.eq(&config.owner) && OWNER_AUTHORIZATIONS.contains(&action_type) {
        return Ok(());
    }

    // Check if the caller has lender authorizations on the vault
    let liquidity_request = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?;
    if let Some(option) = liquidity_request.clone() {
        if let Some(lender) = option.lender {
            if caller.eq(&lender) && LENDER_AUTHORIZATIONS.contains(&action_type) {
                return Ok(());
            }
        }
    }

    // Check if the caller has open authorizations on the vault
    if let Some(option) = liquidity_request {
        if let None = option.lender {
            if OPEN_AUTHORIZATIONS.contains(&action_type) {
                return Ok(());
            }
        }
    }

    Err(ContractError::Unauthorized {})
}