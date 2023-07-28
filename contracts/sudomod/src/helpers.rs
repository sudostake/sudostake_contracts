use crate::state::MIN_VAULT_CODE_UPDATE_INTERVAL;
use crate::{state::CONFIG, ContractError};
use cosmwasm_std::{Addr, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, StdResult, Uint128};

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
    let duration_since_last_update =
        if let Some(vault_code_info_updated_at) = config.vault_code_info_updated_at {
            env.block.time.seconds() - vault_code_info_updated_at.seconds()
        } else {
            MIN_VAULT_CODE_UPDATE_INTERVAL
        };

    if duration_since_last_update < MIN_VAULT_CODE_UPDATE_INTERVAL {
        return Err(ContractError::MinVaultCodeUpdateIntervalNotReached {});
    }

    // Retrun response
    Ok(())
}

pub fn validate_vault_creation_fee(deps: &DepsMut, coins: &[Coin]) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;

    if let Some(vault_creation_fee) = config.vault_creation_fee {
        // Get the actual amount of vault_creation_fee.denom sent by the caller
        let actual_amount = get_amount_for_denom(coins, vault_creation_fee.denom.clone())?;

        if actual_amount != vault_creation_fee.amount.clone() {
            return Err(ContractError::IncorrectTokenCreationFee {
                required: vault_creation_fee.clone(),
                received: Coin {
                    amount: actual_amount,
                    denom: vault_creation_fee.denom,
                },
            });
        }
    }

    Ok(())
}
