use crate::error::ContractError;
use crate::helpers;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VaultCodeListResponse};
use crate::state::{
    Config, VaultCodeInfo, CONFIG, DEFAULT_LIMIT, ENTRY_SEQ, MAX_LIMIT, VAULT_CODE_LIST,
};
use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order,
    Response, StdResult,
};
use cw_storage_plus::Bound;
use std::ops::Add;

// contract info
const CONTRACT_NAME: &str = "sudomod";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

// This tracks the reply from calling the vault contract instantiate submessage
const MINT_VAULT_REPLY_ID: u64 = 1u64;

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
            last_vault_info_update: None,
            vault_creation_fee: None,
        },
    )?;

    // save the entry sequence to storage starting from 0
    ENTRY_SEQ.save(deps.storage, &0u64)?;

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
        ExecuteMsg::SetVaultCodeId { code_id } => {
            execute_set_vault_code_id(deps, env, &info, code_id)
        }
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

pub fn execute_set_vault_code_id(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    code_id: u64,
) -> Result<Response, ContractError> {
    helpers::verify_caller_is_owner(&info, &deps)?;

    // Checks if user can update vault code info
    helpers::can_update_vault_code_info(&deps, &env)?;

    // in order to generate a new seq_id, we get the ENTRY_SEQ and increment it by 1
    let id = ENTRY_SEQ.update::<_, cosmwasm_std::StdError>(deps.storage, |id| Ok(id.add(1)))?;

    // save the new entry to the vault code list
    VAULT_CODE_LIST.save(deps.storage, id, &VaultCodeInfo { id, code_id })?;

    // Update last_vault_info_update
    CONFIG.update(deps.storage, |mut data| -> Result<_, ContractError> {
        data.last_vault_info_update = Some(env.block.time);
        Ok(data)
    })?;

    // return response
    Ok(Response::new()
        .add_attribute("method", "set_vault_code_id")
        .add_attribute("code_id", code_id.to_string())
        .add_attribute("seq_id", id.to_string()))
}

pub fn execute_set_vault_creation_fee(
    deps: DepsMut,
    info: &MessageInfo,
    amount: Coin,
) -> Result<Response, ContractError> {
    helpers::verify_caller_is_owner(&info, &deps)?;

    // Update vault_creation_fee
    CONFIG.update(deps.storage, |mut data| -> Result<_, ContractError> {
        data.vault_creation_fee = Some(amount.clone());
        Ok(data)
    })?;

    // return response
    Ok(Response::new()
        .add_attribute("method", "set_vault_creation_fee")
        .add_attribute("amount", amount.to_string()))
}

pub fn execute_mint_vault(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
) -> Result<Response, ContractError> {
    // verify that caller sends the correct vault_creation_fee
    // add submessage to create a new vault
    // we do not want a reply but it should fail silently in the event
    // of an error in the sub contract

    Ok(Response::new().add_attribute("method", "mint_vault"))
}

pub fn execute_withdraw_balance(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    to_address: Option<String>,
    funds: Coin,
) -> Result<Response, ContractError> {
    helpers::verify_caller_is_owner(&info, &deps)?;

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
    helpers::verify_caller_is_owner(&info, &deps)?;

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
        QueryMsg::QueryVaultCodeList { start_after, limit } => {
            to_binary(&query_vault_code_info_list(deps, start_after, limit)?)
        }
    }
}

pub fn query_info(_deps: Deps) -> StdResult<Config> {
    let config = CONFIG.load(_deps.storage)?;
    Ok(config)
}

fn query_vault_code_info_list(
    deps: Deps,
    start_after: Option<u64>,
    limit: Option<u32>,
) -> StdResult<VaultCodeListResponse> {
    let start = start_after.map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    // Get the entries that matches the range
    let entries: StdResult<Vec<_>> = VAULT_CODE_LIST
        .range(deps.storage, start, None, Order::Ascending)
        .take(limit)
        .collect();

    let results = VaultCodeListResponse {
        entries: entries?.into_iter().map(|l| l.1).collect(),
    };

    Ok(results)
}
