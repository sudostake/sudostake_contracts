use crate::error::ContractError;
use crate::helpers;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{Config, CONFIG};
use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response,
    StdResult,
};

// contract info
const CONTRACT_NAME: &str = "sudomod";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Store the contract name and version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // set the owner as info.sender
    CONFIG.save(
        deps.storage,
        &Config {
            owner: info.sender,
            vault_code_id: None,
            vault_creation_fee: None,
        },
    )?;

    // return response
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SetVaultCodeId { code_id } => execute_set_vault_code_id(deps, &info, code_id),
        ExecuteMsg::SetVaultCreationFee { amount } => {
            execute_set_vault_creation_fee(deps, &info, amount)
        }
        ExecuteMsg::MintVault {} => execute_mint_vault(deps, env, &info),
        ExecuteMsg::WithdrawBalance { to_address, funds } => {
            execute_withdraw_balance(deps, env, &info, to_address, funds)
        }
        ExecuteMsg::TransferOwnership { to_address } => {
            execute_transfer_ownership(deps, &info, to_address)
        }
    }
}

fn verify_caller_is_owner(info: &MessageInfo, deps: &DepsMut) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    let is_owner = info.sender.eq(&config.owner);

    if !is_owner {
        return Err(ContractError::Unauthorized {});
    }

    Ok(())
}

pub fn execute_set_vault_code_id(
    deps: DepsMut,
    info: &MessageInfo,
    code_id: u64,
) -> Result<Response, ContractError> {
    verify_caller_is_owner(&info, &deps)?;

    // Set code_id to be used for creating new instances of vaults
    CONFIG.update(deps.storage, |mut data| -> Result<_, ContractError> {
        data.vault_code_id = Some(code_id);
        Ok(data)
    })?;

    // return response
    Ok(Response::new().add_attribute("method", "set_vault_code_id"))
}

pub fn execute_set_vault_creation_fee(
    deps: DepsMut,
    info: &MessageInfo,
    amount: Coin,
) -> Result<Response, ContractError> {
    verify_caller_is_owner(&info, &deps)?;

    // Set vault creation fee to be paid by users who wants to create a new vault
    CONFIG.update(deps.storage, |mut data| -> Result<_, ContractError> {
        data.vault_creation_fee = Some(amount);
        Ok(data)
    })?;

    // return response
    Ok(Response::new().add_attribute("method", "set_vault_creation_fee"))
}

pub fn execute_mint_vault(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
) -> Result<Response, ContractError> {
    // TODO
    let mut response = Response::new();

    // return response
    Ok(response.add_attribute("method", "mint_vault"))
}

pub fn execute_withdraw_balance(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    to_address: Option<String>,
    funds: Coin,
) -> Result<Response, ContractError> {
    verify_caller_is_owner(&info, &deps)?;

    // Check if the contract balance is >= the requested amount to withdraw
    let available_balance = helpers::get_amount_for_denom(
        &deps
            .querier
            .query_all_balances(env.contract.address.clone())?,
        funds.denom.clone(),
    )?;
    if available_balance < funds.amount {
        return Err(ContractError::InsufficientBalance {
            available: Coin {
                amount: available_balance,
                denom: funds.denom.clone(),
            },
            required: funds,
        });
    }

    // Get the recipient to send funds to
    let recipient: Addr = if let Some(val) = to_address {
        deps.api.addr_validate(&val)?
    } else {
        let config = CONFIG.load(deps.storage)?;
        config.owner
    };

    // Respond
    Ok(Response::new()
        .add_message(helpers::get_bank_transfer_to_msg(
            &recipient,
            &funds.denom,
            funds.amount,
        ))
        .add_attributes(vec![
            attr("method", "withdraw_balance"),
            attr("recipient", recipient.to_string()),
        ]))
}

pub fn execute_transfer_ownership(
    deps: DepsMut,
    info: &MessageInfo,
    to_address: String,
) -> Result<Response, ContractError> {
    verify_caller_is_owner(&info, &deps)?;

    // validate the new owner_address
    let new_owner = deps.api.addr_validate(&to_address)?;

    // Set the new owner of this vault
    CONFIG.update(deps.storage, |mut data| -> Result<_, ContractError> {
        data.owner = new_owner;
        Ok(data)
    })?;

    Ok(Response::new().add_attributes(vec![
        attr("method", "transfer_ownership"),
        attr("to_address", to_address.to_string()),
    ]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Info {} => to_binary(&query_info(deps)?),
    }
}

pub fn query_info(_deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(_deps.storage)?;
    Ok(config)
}
