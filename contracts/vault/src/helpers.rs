use crate::{state::OPEN_LIQUIDITY_REQUEST, ContractError};
use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, StdResult, Uint128};

pub fn verify_validator_is_active(deps: &DepsMut, validator: &str) -> Result<(), ContractError> {
    let res = deps.querier.query_validator(validator)?;

    if res.is_none() {
        return Err(ContractError::ValidatorIsInactive {
            validator: validator.to_string(),
        });
    }

    Ok(())
}

pub fn get_available_staking_balace(
    env: &Env,
    deps: &DepsMut,
    denom_str: String,
) -> Result<Coin, ContractError> {
    // find the coin with non-zero balance that matches the denom
    let contract_balances = deps
        .querier
        .query_all_balances(env.contract.address.clone())?;

    let coin = contract_balances
        .iter()
        .find(|coin| coin.denom == denom_str);

    Ok(match coin {
        Some(coin) => coin.clone(),
        None => Coin {
            amount: Uint128::zero(),
            denom: denom_str,
        },
    })
}

pub fn query_total_delegations(deps: &DepsMut, env: &Env) -> StdResult<Uint128> {
    let total = deps
        .querier
        .query_all_delegations(env.contract.address.clone())?
        .iter()
        .map(|d| d.amount.amount)
        .sum();

    Ok(total)
}

pub fn validate_amount_to_delegate(
    env: &Env,
    deps: &DepsMut,
    amount_to_delegate: Uint128,
    denom_str: String,
) -> Result<(), ContractError> {
    let balance = get_available_staking_balace(env, deps, denom_str.clone())?;

    if balance.amount < amount_to_delegate {
        return Err(ContractError::InsufficientBalance {
            available: balance,
            required: Coin {
                denom: denom_str,
                amount: amount_to_delegate,
            },
        });
    }

    Ok(())
}

pub fn get_bank_transfer_to_msg(recipient: &Addr, denom: &str, amount: Uint128) -> CosmosMsg {
    let transfer_bank_msg = BankMsg::Send {
        to_address: recipient.into(),
        amount: vec![Coin {
            denom: denom.into(),
            amount,
        }],
    };

    let transfer_bank_cosmos_msg: CosmosMsg = transfer_bank_msg.into();
    transfer_bank_cosmos_msg
}

pub fn has_open_liquidity_request(deps: &DepsMut) -> StdResult<bool> {
    let data = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?;
    Ok(data.is_some())
}
