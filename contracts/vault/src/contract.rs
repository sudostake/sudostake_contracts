use crate::authorisation::{authorize, ActionTypes};
use crate::error::ContractError;
use crate::helpers::{
    has_open_liquidity_request, query_total_delegations, validate_amount_to_delegate,
    verify_validator_is_active,
};
use crate::msg::{ExecuteMsg, InfoResponse, InstantiateMsg, LiquidityRequestOptionMsg, QueryMsg};
use crate::state::{ActiveOption, Config, OPEN_LIQUIDITY_REQUEST, CONFIG};
use cosmwasm_std::{
    attr, entry_point, to_binary, Binary, Coin, Deps, DepsMut, DistributionMsg, Env, MessageInfo,
    Response, StakingMsg, StdResult, Uint128, VoteOption,
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

    // Validate the owner_address
    let owner = deps.api.addr_validate(&msg.owner_address)?;

    // Validate account_manager_address
    let acc_manager = deps.api.addr_validate(&msg.account_manager_address)?;

    // Save contract state
    CONFIG.save(deps.storage, &Config { owner, acc_manager })?;

    // Init OPEN_LIQUIDITY_REQUEST to None
    OPEN_LIQUIDITY_REQUEST.save(deps.storage, &None)?;

    // response
    Ok(Response::new().add_attribute("method", "instantiate"))
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match _msg {
        ExecuteMsg::Delegate { validator, amount } => {
            authorize(
                &deps,
                _info.sender.clone(),
                ActionTypes::Delegate(has_open_liquidity_request(&deps)?),
            )?;
            execute_delegate(deps, _env, &_info, validator, amount)
        }

        ExecuteMsg::Undelegate { validator, amount } => {
            authorize(
                &deps,
                _info.sender.clone(),
                ActionTypes::Undelegate(has_open_liquidity_request(&deps)?),
            )?;
            execute_undelegate(deps, _env, &_info, validator, amount)
        }
        ExecuteMsg::OpenLRO { option } => {
            authorize(
                &deps,
                _info.sender.clone(),
                ActionTypes::OpenLRO(has_open_liquidity_request(&deps)?),
            )?;
            execute_open_liquidity_request(deps, _env, option)
        }
        ExecuteMsg::ClosePendingLRO {} => {
            authorize(
                &deps,
                _info.sender.clone(),
                ActionTypes::ClosePendingLRO(has_open_liquidity_request(&deps)?),
            )?;
            execute_close_pending_lro(deps, _env, &_info)
        }
        ExecuteMsg::Vote { proposal_id, vote } => {
            authorize(
                &deps,
                _info.sender.clone(),
                ActionTypes::Vote(has_open_liquidity_request(&deps)?),
            )?;
            execute_vote(deps, _env, &_info, proposal_id, vote)
        }
        ExecuteMsg::ClaimDelegatorRewards {} => {
            authorize(
                &deps,
                _info.sender.clone(),
                ActionTypes::ClaimDelegatorRewards {},
            )?;
            execute_claim_delegator_rewards(deps, _env)
        }
        ExecuteMsg::LiquidateCollateral {} => {
            authorize(
                &deps,
                _info.sender.clone(),
                ActionTypes::LiquidateCollateral {},
            )?;
            execute_liquidate_collateral(deps, _env, &_info)
        }
        ExecuteMsg::TransferOwnership { to_address } => {
            authorize(
                &deps,
                _info.sender.clone(),
                ActionTypes::TransferOwnership {},
            )?;
            execute_transfer_ownership(deps, _env, &_info, to_address)
        }
        ExecuteMsg::Redelegate {
            src_validator,
            dst_validator,
            amount,
        } => {
            authorize(&deps, _info.sender.clone(), ActionTypes::Redelegate {})?;
            execute_redelegate(deps, _env, &_info, src_validator, dst_validator, amount)
        }

        ExecuteMsg::AcceptLRO { is_contract_user } => {
            authorize(&deps, _info.sender.clone(), ActionTypes::AcceptLRO {})?;
            execute_accept_lro(deps, _env, &_info, is_contract_user)
        }

        ExecuteMsg::RepayLoan {} => {
            authorize(&deps, _info.sender.clone(), ActionTypes::RepayLoan {})?;
            execute_repay_loan(deps, _env, &_info)
        }
        ExecuteMsg::WithdrawBalance { to_address, funds } => {
            authorize(&deps, _info.sender.clone(), ActionTypes::WithdrawBalance {})?;
            execute_withdraw_balance(deps, _env, &_info, to_address, funds)
        }
    }
}

pub fn execute_delegate(
    deps: DepsMut,
    env: Env,
    _info: &MessageInfo,
    validator: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    verify_validator_is_active(&deps, validator.as_str())?;

    // Validate amount to delegate is not above availabe contract balance
    let denom_str = deps.querier.query_bonded_denom()?;
    validate_amount_to_delegate(&env, &deps, amount, denom_str.clone())?;

    // Create sdk_msg for staking tokens
    let sdk_msg = StakingMsg::Delegate {
        validator: validator.clone(),
        amount: Coin {
            denom: denom_str,
            amount,
        },
    };

    // Respond
    Ok(Response::new().add_message(sdk_msg).add_attributes(vec![
        attr("method", "delegate"),
        attr("amount", amount.to_string()),
        attr("validator", validator),
    ]))
}

pub fn execute_redelegate(
    deps: DepsMut,
    _env: Env,
    _info: &MessageInfo,
    src_validator: String,
    dst_validator: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    verify_validator_is_active(&deps, dst_validator.as_str())?;

    // Create sdk_msg for un-staking tokens
    let denom_str = deps.querier.query_bonded_denom()?;
    let sdk_msg = StakingMsg::Redelegate {
        src_validator: src_validator.clone(),
        dst_validator: dst_validator.clone(),
        amount: Coin {
            denom: denom_str,
            amount,
        },
    };

    // Respond
    Ok(Response::new().add_message(sdk_msg).add_attributes(vec![
        attr("method", "redelegate"),
        attr("amount", amount.to_string()),
        attr("src_validator", src_validator),
        attr("dst_validator", dst_validator),
    ]))
}

pub fn execute_undelegate(
    deps: DepsMut,
    env: Env,
    _info: &MessageInfo,
    validator: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let validator_delegation = deps
        .querier
        .query_delegation(env.contract.address.clone(), validator.clone())
        .unwrap();

    // Verify amount <= validator_delegation
    if validator_delegation.is_some() {
        let data = validator_delegation.unwrap();
        if amount > data.amount.amount {
            return Err(ContractError::MaxUndelegateAmountExceeded {
                amount,
                validator_delegation: data.amount.amount,
            });
        }
    }

    // Create sdk_msg for un-staking tokens
    let denom_str = deps.querier.query_bonded_denom()?;
    let sdk_msg = StakingMsg::Undelegate {
        validator: validator.clone(),
        amount: Coin {
            denom: denom_str,
            amount,
        },
    };

    // Respond
    Ok(Response::new().add_message(sdk_msg).add_attributes(vec![
        attr("method", "undelegate"),
        attr("amount", amount.to_string()),
        attr("validator", validator),
    ]))
}

pub fn execute_open_liquidity_request(
    deps: DepsMut,
    env: Env,
    option: LiquidityRequestOptionMsg,
) -> Result<Response, ContractError> {
    // Validate liquidity request option
    match option.clone() {
        LiquidityRequestOptionMsg::FixedInterestRental {
            requested_amount,
            claimable_tokens,
            is_lp_group: _,
            can_cast_vote: _,
        } => {
            if requested_amount.amount.is_zero() || claimable_tokens.is_zero() {
                return Err(ContractError::InvalidLiquidityRequestOption {});
            }
        }

        LiquidityRequestOptionMsg::FixedTermRental {
            requested_amount,
            duration_in_seconds,
            is_lp_group: _,
            can_cast_vote: _,
        } => {
            if requested_amount.amount.is_zero() || duration_in_seconds == 0u64 {
                return Err(ContractError::InvalidLiquidityRequestOption {});
            }
        }

        LiquidityRequestOptionMsg::FixedTermLoan {
            requested_amount,
            duration_in_seconds,
            collateral_amount,
            can_claim_rewards: _,
            is_lp_group: _,
            can_cast_vote: _,
        } => {
            if query_total_delegations(&deps, &env)? < collateral_amount
                || requested_amount.amount.is_zero()
                || duration_in_seconds == 0u64
            {
                return Err(ContractError::InvalidLiquidityRequestOption {});
            }
        }
    };

    // Save liquidity request to state
    OPEN_LIQUIDITY_REQUEST.save(
        deps.storage,
        &Some(ActiveOption {
            lender: None,
            state: None,
            msg: option,
        }),
    )?;

    // Respond
    Ok(Response::new().add_attributes(vec![attr("method", "open_liquidity_request")]))
}

pub fn execute_close_pending_lro(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
) -> Result<Response, ContractError> {
    // if active liquidity reqest already has a lender connected, we return error
    // else we clear the pending liquidity request
    // respond
    Ok(Response::default())
}

pub fn execute_accept_lro(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    is_contract_user: Option<bool>,
) -> Result<Response, ContractError> {
    // TODO implement this
    // respond
    Ok(Response::default())
}

pub fn execute_claim_delegator_rewards(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    // Update distribute_msgs and total_rewards_to_claim
    let mut distribute_msgs = vec![];
    let mut total_rewards_to_claim = Uint128::new(0);
    deps.querier
        .query_all_delegations(env.contract.address.clone())?
        .iter()
        .for_each(|d| {
            distribute_msgs.push(DistributionMsg::WithdrawDelegatorReward {
                validator: d.validator.clone(),
            });

            // Update total_rewards_to_claim
            deps.querier
                .query_delegation(env.contract.address.clone(), d.validator.clone())
                .unwrap()
                .unwrap()
                .accumulated_rewards
                .iter()
                .for_each(|c| total_rewards_to_claim += c.amount);
        });

    // TODO
    // here we distribute rewards allocations

    // respond
    Ok(Response::new()
        .add_messages(distribute_msgs)
        .add_attributes(vec![
            attr("method", "claim_delegator_rewards"),
            attr("total_rewards_to_claim", total_rewards_to_claim.to_string()),
        ]))
}

pub fn execute_liquidate_collateral(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
) -> Result<Response, ContractError> {
    // TODO implement this˝
    // respond
    Ok(Response::default())
}

pub fn execute_repay_loan(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
) -> Result<Response, ContractError> {
    // TODO implement this˝
    // respond
    Ok(Response::default())
}

pub fn execute_withdraw_balance(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    to_address: Option<String>,
    funds: Coin,
) -> Result<Response, ContractError> {
    // TODO implement this
    // respond
    Ok(Response::default())
}

pub fn execute_transfer_ownership(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    to_address: String,
) -> Result<Response, ContractError> {
    // TODO implement this
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
    // TODO
    // use cosmwasm_std::{GovMsg, VoteOption};
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
    let config = CONFIG.load(_deps.storage)?;
    let liquidity_request = OPEN_LIQUIDITY_REQUEST.load(_deps.storage)?;
    Ok(InfoResponse {
        config,
        liquidity_request,
    })
}
