use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InfoResponse, InstantiateMsg, LiquidityRequestOption, QueryMsg};
use crate::state::{Config, CONFIG};
use cosmwasm_std::{
    entry_point, to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128, VoteOption,
};

// contract info
pub const CONTRACT_NAME: &str = "vault_contract";
pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Store the contract name and version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // make sure the staking_denom is a non-empty string
    if msg.staking_denom.is_empty() {
        return Err(ContractError::InvalidStakingDenom {});
    }

    // Validate the owner_address
    let address = deps.api.addr_validate(&msg.owner_address)?;

    // Save contract state
    CONFIG.save(
        deps.storage,
        &Config {
            staking_denom: msg.staking_denom,
            owner: address,
        },
    )?;

    // response
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match _msg {
        ExecuteMsg::Delegate { validator, amount } => {
            execute_delegate(_deps, _env, &_info, validator, amount)
        }
        ExecuteMsg::Undelegate { validator, amount } => {
            execute_undelegate(_deps, _env, &_info, validator, amount)
        }
        ExecuteMsg::Redelegate {
            src_validator,
            dst_validator,
            amount,
        } => execute_redelegate(_deps, _env, &_info, src_validator, dst_validator, amount),
        ExecuteMsg::ClaimDelegatorRewards { withdraw } => {
            execute_claim_delegator_rewards(_deps, _env, &_info, withdraw)
        }
        ExecuteMsg::OpenLRO { option } => execute_open_lro(_deps, _env, &_info, option),
        ExecuteMsg::ClosePendingLRO {} => execute_close_pending_lro(_deps, _env, &_info),
        ExecuteMsg::AcceptLRO { is_contract_user } => {
            execute_accept_lro(_deps, _env, &_info, is_contract_user)
        }
        ExecuteMsg::ProcessClaimsForLRO {} => execute_process_claims_for_lro(_deps, _env, &_info),
        ExecuteMsg::Withdraw { to_address, funds } => {
            execute_withdraw(_deps, _env, &_info, to_address, funds)
        }
        ExecuteMsg::TransferOwnership { to_address } => {
            execute_transfer_ownership(_deps, _env, &_info, to_address)
        }
        ExecuteMsg::Vote { proposal_id, vote } => {
            execute_vote(_deps, _env, &_info, proposal_id, vote)
        }
    }
}

pub fn execute_delegate(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    validator: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // todo
    // verify vault owner that is calling this method
    // verify the correct amount and denom was sent along to be staked
    // increase the current total amount staked in the contract state
    // create sdk_msg for staking tokens
    // respond

    Ok(Response::default())
}

pub fn execute_undelegate(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    validator: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // TODO implement this˝
    // respond
    Ok(Response::default())
}

pub fn execute_redelegate(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    src_validator: String,
    dst_validator: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    // TODO implement this˝
    // respond
    Ok(Response::default())
}

pub fn execute_claim_delegator_rewards(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    withdraw: Option<bool>,
) -> Result<Response, ContractError> {
    // TODO implement this˝
    // respond
    Ok(Response::default())
}

pub fn execute_open_lro(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    option: LiquidityRequestOption,
) -> Result<Response, ContractError> {
    // TODO implement this˝
    // respond
    Ok(Response::default())
}

pub fn execute_close_pending_lro(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
) -> Result<Response, ContractError> {
    // TODO implement this˝
    // respond
    Ok(Response::default())
}

pub fn execute_accept_lro(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    is_contract_user: Option<bool>,
) -> Result<Response, ContractError> {
    // TODO implement this˝
    // respond
    Ok(Response::default())
}

pub fn execute_process_claims_for_lro(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
) -> Result<Response, ContractError> {
    // TODO implement this˝
    // respond
    Ok(Response::default())
}

pub fn execute_withdraw(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    to_address: Option<String>,
    funds: Coin,
) -> Result<Response, ContractError> {
    // TODO implement this˝
    // respond
    Ok(Response::default())
}

pub fn execute_transfer_ownership(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    to_address: String,
) -> Result<Response, ContractError> {
    // TODO implement this˝
    // respond
    Ok(Response::default())
}

pub fn execute_vote(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    proposal_id: u64,
    vote: VoteOption,
) -> Result<Response, ContractError> {
    // TODO implement this˝
    // respond
    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Info {} => to_binary(&query_info(deps)?),
    }
}

pub fn query_info(_deps: Deps) -> StdResult<InfoResponse> {
    Ok(InfoResponse {})
}
