use crate::{
    state::{ActiveOption, CONFIG, OPEN_LIQUIDITY_REQUEST},
    ContractError,
};
use cosmwasm_std::{Addr, DepsMut};

// True  = vault has an open liquidity request
// False = vault does not have an open liquidity request
type WithOpenLiquidityRequest = bool;

#[derive(PartialEq)]
pub enum ActionTypes {
    Delegate(WithOpenLiquidityRequest),
    Redelegate,
    Undelegate(WithOpenLiquidityRequest),
    OpenLiquidityRequest(WithOpenLiquidityRequest),
    CloseLiquidityRequest(WithOpenLiquidityRequest),
    WithdrawBalance(WithOpenLiquidityRequest),
    TransferOwnership,
    AcceptLiquidityRequest,
    ClaimDelegatorRewards,
    LiquidateCollateral(WithOpenLiquidityRequest),
    RepayLoan(WithOpenLiquidityRequest),
    Vote,
}

// Applies to the owner of the vault
const OWNER_AUTHORIZATIONS: [ActionTypes; 11] = [
    ActionTypes::Delegate(false),
    ActionTypes::Redelegate,
    ActionTypes::Undelegate(false),
    ActionTypes::OpenLiquidityRequest(false),
    ActionTypes::CloseLiquidityRequest(true),
    ActionTypes::TransferOwnership,
    ActionTypes::RepayLoan(true),
    ActionTypes::ClaimDelegatorRewards,
    ActionTypes::LiquidateCollateral(true),
    ActionTypes::WithdrawBalance(false),
    ActionTypes::Vote,
];

// Applies to the active lenders on the vault
const LENDER_AUTHORIZATIONS: [ActionTypes; 3] = [
    ActionTypes::ClaimDelegatorRewards,
    ActionTypes::LiquidateCollateral(true),
    ActionTypes::Vote,
];

// Applies to all users trying to lend to the open liquidity request option
const OPEN_AUTHORIZATIONS: [ActionTypes; 1] = [ActionTypes::AcceptLiquidityRequest];

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
    if let Some(ActiveOption {
        lender: Some(lender),
        state: Some(_state),
        msg: _,
    }) = liquidity_request
    {
        if caller.eq(&lender) && LENDER_AUTHORIZATIONS.contains(&action_type) {
            return Ok(());
        }
    }

    // Check if the caller has open authorizations on the vault
    let liquidity_request = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?;
    if let Some(ActiveOption {
        lender: None,
        state: None,
        msg: _,
    }) = liquidity_request
    {
        if OPEN_AUTHORIZATIONS.contains(&action_type) {
            return Ok(());
        }
    }

    Err(ContractError::Unauthorized {})
}
