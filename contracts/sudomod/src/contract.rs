use crate::error::ContractError;
use crate::helpers;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, VaultCodeListResponse};
use crate::state::{
    Config, VaultCodeInfo, CONFIG, VAULT_CODE_LIST, VAULT_CODE_SEQ, VAULT_INSTANTIATION_SEQ,
};
use crate::state::{CONTRACT_NAME, CONTRACT_VERSION, DEFAULT_LIMIT, MAX_LIMIT};
use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Order,
    ReplyOn, Response, StdResult, SubMsg, WasmMsg,
};
use cw_storage_plus::Bound;
use std::ops::Add;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Store the contract name and version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Set owner as info.sender
    CONFIG.save(
        deps.storage,
        &Config {
            owner: info.sender,
            vault_code_info_updated_at: None,
            vault_creation_fee: None,
        },
    )?;

    // Set VAULT_CODE_SEQ to 0
    VAULT_CODE_SEQ.save(deps.storage, &0u64)?;

    // Set VAULT_INSTANTIATION_SEQ to 0
    VAULT_INSTANTIATION_SEQ.save(deps.storage, &0u64)?;

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
        ExecuteMsg::MintVault {} => execute_mint_vault(deps, &info),
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

    // Get a new seq_id for vault code update
    let seq_id =
        VAULT_CODE_SEQ.update::<_, cosmwasm_std::StdError>(deps.storage, |id| Ok(id.add(1)))?;

    // save the new entry to the vault code list
    VAULT_CODE_LIST.save(
        deps.storage,
        seq_id,
        &VaultCodeInfo {
            id: seq_id,
            code_id,
        },
    )?;

    // Update vault_code_info_updated_at
    CONFIG.update(deps.storage, |mut data| -> Result<_, ContractError> {
        data.vault_code_info_updated_at = Some(env.block.time);
        Ok(data)
    })?;

    // return response
    Ok(Response::new()
        .add_attribute("method", "set_vault_code_id")
        .add_attribute("code_id", code_id.to_string())
        .add_attribute("seq_id", seq_id.to_string()))
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

pub fn execute_mint_vault(deps: DepsMut, info: &MessageInfo) -> Result<Response, ContractError> {
    // Verify that VAULT_CODE_SEQ for setting vault_code_id is greater than 0
    // Which indicates there is a vault_code_id already set
    let vault_code_seq_id = VAULT_CODE_SEQ.load(deps.storage)?;
    if vault_code_seq_id.eq(&0u64) {
        return Err(ContractError::VaultCodeIdNotSet {});
    }

    // Verify that caller sends the correct vault_creation_fee
    // If no vault creation fee is set, it implies that this vault
    // instance will be created for free.
    helpers::validate_vault_creation_fee(&deps, &info.funds)?;

    // Get a new vault_instance_seq_id
    let vault_instance_seq_id = VAULT_INSTANTIATION_SEQ
        .update::<_, cosmwasm_std::StdError>(deps.storage, |id| Ok(id.add(1)))?;

    // Add submessage to create a new vault
    let latest_code_info = VAULT_CODE_LIST.load(deps.storage, vault_code_seq_id)?;
    let instantiate_vault_sub_msg = SubMsg {
        gas_limit: None,
        id: 0u64,
        reply_on: ReplyOn::Never,
        msg: WasmMsg::Instantiate {
            admin: None,
            code_id: latest_code_info.code_id,
            msg: to_binary(&vault_contract::msg::InstantiateMsg {
                owner_address: info.sender.to_string(),
                from_code_id: latest_code_info.code_id,
                index_number: vault_instance_seq_id,
            })?,
            funds: vec![],
            label: format!("Vault Number {:?}", vault_instance_seq_id),
        }
        .into(),
    };

    // return response
    Ok(Response::new()
        .add_submessage(instantiate_vault_sub_msg)
        .add_attribute("method", "mint_vault")
        .add_attribute("vault_code_id", latest_code_info.code_id.to_string())
        .add_attribute("vault_instance_seq_id", vault_instance_seq_id.to_string()))
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
