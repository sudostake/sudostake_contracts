use crate::{
    state::{CONFIG, LIQUIDITY_REQUEST_STATE},
    types::{ActionTypes, ActiveOption, LiquidityRequestStatus},
    ContractError,
};
use cosmwasm_std::{Addr, DepsMut};

// Applies to owner of vault
const OWNER_AUTHORIZATIONS: [ActionTypes; 12] = [
    ActionTypes::Delegate,
    ActionTypes::Redelegate,
    ActionTypes::Undelegate(LiquidityRequestStatus::Closed),
    ActionTypes::RequestLiquidity(LiquidityRequestStatus::Closed),
    ActionTypes::AcceptCounterOffer(LiquidityRequestStatus::Pending),
    ActionTypes::ClosePendingLiquidityRequest(LiquidityRequestStatus::Pending),
    ActionTypes::TransferOwnership,
    ActionTypes::RepayLoan(LiquidityRequestStatus::Active),
    ActionTypes::ClaimDelegatorRewards,
    ActionTypes::LiquidateCollateral(LiquidityRequestStatus::Active),
    ActionTypes::WithdrawBalance,
    ActionTypes::Vote,
];

// Applies to the active lenders on the vault
const LENDER_AUTHORIZATIONS: [ActionTypes; 4] = [
    ActionTypes::Redelegate,
    ActionTypes::ClaimDelegatorRewards,
    ActionTypes::LiquidateCollateral(LiquidityRequestStatus::Active),
    ActionTypes::Vote,
];

// Applies to any user trying to lend to the pending liquidity request option
const OPEN_AUTHORIZATIONS: [ActionTypes; 4] = [
    ActionTypes::AcceptLiquidityRequest(LiquidityRequestStatus::Pending),
    ActionTypes::OpenCounterOffer(LiquidityRequestStatus::Pending),
    ActionTypes::UpdateCounterOffer(LiquidityRequestStatus::Pending),
    ActionTypes::CancelCounterOffer(LiquidityRequestStatus::Pending),
];

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
    let liquidity_request = LIQUIDITY_REQUEST_STATE.load(deps.storage)?;
    if let Some(ActiveOption {
        lender: Some(lender),
        ..
    }) = liquidity_request
    {
        if caller.eq(&lender) && LENDER_AUTHORIZATIONS.contains(&action_type) {
            return Ok(());
        }
    }

    // Check if the caller has open authorizations on the vault
    if caller.ne(&config.owner) && OPEN_AUTHORIZATIONS.contains(&action_type) {
        return Ok(());
    }

    Err(ContractError::Unauthorized {})
}
