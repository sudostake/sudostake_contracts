use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};
use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response};

// contract info
pub const CONTRACT_NAME: &str = "accounts_manager_contract";
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
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    // return response
    Ok(Response::new())
}

// TODO add query
