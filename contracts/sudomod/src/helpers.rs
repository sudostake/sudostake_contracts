use crate::{state::CONFIG, ContractError};
use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, StdResult, Uint128};

// This is the minimum duration between calls to SetVaultCodeId
const MIN_VAULT_CODE_UPDATE_INTERVAL: u64 = 60 * 60 * 24 * 30;

pub fn get_bank_transfer_to_msg(recipient: &Addr, denom: &str, amount: Uint128) -> CosmosMsg {
    BankMsg::Send {
        to_address: recipient.into(),
        amount: vec![Coin {
            denom: denom.into(),
            amount,
        }],
    }
    .into()
}

pub fn get_amount_for_denom(funds: &[Coin], denom_str: String) -> StdResult<Uint128> {
    Ok(funds
        .iter()
        .filter(|c| c.denom == denom_str)
        .map(|c| c.amount)
        .sum())
}

pub fn verify_caller_is_owner(info: &MessageInfo, deps: &DepsMut) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let is_owner = info.sender.eq(&config.owner);

    if !is_owner {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

pub fn can_update_vault_code_info(deps: &DepsMut, env: &Env) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;

    // Calculate duration_since_last_update
    let duration_since_last_update = if config.last_vault_info_update.is_some() {
        env.block.time.seconds() - config.last_vault_info_update.unwrap().seconds()
    } else {
        MIN_VAULT_CODE_UPDATE_INTERVAL
    };

    if duration_since_last_update < MIN_VAULT_CODE_UPDATE_INTERVAL {
        return Err(ContractError::MinVaultCodeUpdateIntervalNotReached {});
    }

    // Retrun response
    Ok(())
}
