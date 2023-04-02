use cosmwasm_std::{Addr, DepsMut};

use crate::{
    state::{ACTIVE_LRO, CONFIG},
    ContractError,
};

#[derive(PartialEq)]
/// The optional bool argument signifies weather the action
/// is available when there is an active lro
pub enum ActionTypes {
    Delegate(bool),
    Redelegate,
    Undelegate(bool),
    OpenLRO(bool),
    ClosePendingLRO,
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
    ActionTypes::ClosePendingLRO,
    ActionTypes::WithdrawBalance,
    ActionTypes::TransferOwnership,
    ActionTypes::ClaimDelegatorRewards,
    ActionTypes::LiquidateCollateral,
    ActionTypes::RepayLoan,
    ActionTypes::Vote(false),
];

const LENDER_AUTHORIZATIONS: [ActionTypes; 5] = [
    ActionTypes::AcceptLRO,
    ActionTypes::ClaimDelegatorRewards,
    ActionTypes::LiquidateCollateral,
    ActionTypes::RepayLoan,
    ActionTypes::Vote(true),
];

// ActionTypes::Delegate(helpers.has_active_lro())
pub fn _authorize(
    deps: &DepsMut,
    caller: Addr,
    action_type: ActionTypes,
) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let active_lro = ACTIVE_LRO.load(deps.storage)?;

    if caller.eq(&config.owner) && OWNER_AUTHORIZATIONS.contains(&action_type) {
        return Ok(());
    }

    if let Some(option) = active_lro {
        if caller.eq(&option.lender) && LENDER_AUTHORIZATIONS.contains(&action_type) {
            return Ok(());
        }
    }

    Err(ContractError::Unauthorized {})
}
