use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InfoResponse, InstantiateMsg, QueryMsg};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};

// contract info
pub const CONTRACT_NAME: &str = "sudomod";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Store the contract name and version
    cw2::set_contract_version(_deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // return response
    Ok(Response::new())
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
            execute_set_vault_creation_fee(deps, env, &info, amount)
        }
        ExecuteMsg::MintVault {} => execute_mint_vault(deps, env, &info),
        ExecuteMsg::WithdrawBalance { to_address, funds } => {
            execute_withdraw_balance(deps, env, &info, to_address, funds)
        }
        ExecuteMsg::TransferOwnership { to_address } => {
            execute_transfer_ownership(deps, env, &info, to_address)
        }
    }
}

pub fn execute_set_vault_code_id(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    code_id: u64,
) -> Result<Response, ContractError> {
    let mut response = Response::new();

    // respond
    Ok(response.add_attribute("method", "set_vault_code_id"))
}

pub fn execute_set_vault_creation_fee(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    amount: Coin,
) -> Result<Response, ContractError> {
    let mut response = Response::new();

    // respond
    Ok(response.add_attribute("method", "set_vault_creation_fee"))
}

pub fn execute_mint_vault(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
) -> Result<Response, ContractError> {
    let mut response = Response::new();

    // respond
    Ok(response.add_attribute("method", "mint_vault"))
}

pub fn execute_withdraw_balance(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    to_address: Option<String>,
    funds: Coin,
) -> Result<Response, ContractError> {
    let mut response = Response::new();

    // respond
    Ok(response.add_attribute("method", "withdraw_balance"))
}

pub fn execute_transfer_ownership(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    to_address: String,
) -> Result<Response, ContractError> {
    let mut response = Response::new();

    // respond
    Ok(response.add_attribute("method", "transfer_ownership"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Info {} => to_binary(&query_info(deps)?),
    }
}

pub fn query_info(_deps: Deps) -> StdResult<InfoResponse> {
    Ok(InfoResponse {})
}
