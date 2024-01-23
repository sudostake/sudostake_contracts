use crate::authorisation::{authorize, ActionTypes};
use crate::error::ContractError;
use crate::helpers;
use crate::msg::{
    AllDelegationsResponse, ExecuteMsg, InfoResponse, InstantiateMsg, QueryMsg, StakingInfoResponse,
};
use crate::state::{
    ActiveOption, Config, LiquidityRequestMsg, LiquidityRequestState, CONFIG, CONTRACT_NAME,
    CONTRACT_VERSION, INSTANTIATOR_ADDR, OPEN_LIQUIDITY_REQUEST, STAKE_LIQUIDATION_INTERVAL,
};
use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, Binary, Coin, Deps, DepsMut, Env, GovMsg, MessageInfo,
    Response, StakingMsg, StdResult, Uint128, VoteOption,
};

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    // Verify that info.sender.eq(INSTANTIATOR_ADDR)
    if _info.sender.to_string().ne(INSTANTIATOR_ADDR) {
        return Err(ContractError::Unauthorized {});
    }

    // Validate the owner_address
    let owner = deps.api.addr_validate(&msg.owner_address)?;

    // Store the contract name and version
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    // Save contract state
    OPEN_LIQUIDITY_REQUEST.save(deps.storage, &None)?;
    CONFIG.save(
        deps.storage,
        &Config {
            owner,
            from_code_id: msg.from_code_id,
            index_number: msg.index_number,
        },
    )?;

    // Respond
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
            authorize(&deps, _info.sender.clone(), ActionTypes::Delegate {})?;
            execute_delegate(deps, _env, &_info, validator, amount)
        }

        ExecuteMsg::Redelegate {
            src_validator,
            dst_validator,
            amount,
        } => {
            authorize(&deps, _info.sender.clone(), ActionTypes::Redelegate {})?;
            execute_redelegate(deps, _env, &_info, src_validator, dst_validator, amount)
        }

        ExecuteMsg::Undelegate { validator, amount } => {
            let action_type = ActionTypes::Undelegate(helpers::has_open_liquidity_request(&deps)?);
            authorize(&deps, _info.sender.clone(), action_type)?;
            execute_undelegate(deps, _env, &_info, validator, amount)
        }

        ExecuteMsg::RequestLiquidity { option } => {
            let action_type =
                ActionTypes::RequestLiquidity(helpers::has_open_liquidity_request(&deps)?);
            authorize(&deps, _info.sender.clone(), action_type)?;
            execute_request_liquidity(deps, _env, option)
        }

        ExecuteMsg::ClosePendingLiquidityRequest {} => {
            let action_type = ActionTypes::ClosePendingLiquidityRequest(
                helpers::has_open_liquidity_request(&deps)?,
            );
            authorize(&deps, _info.sender.clone(), action_type)?;
            execute_close_pending_liquidity_request(deps)
        }

        ExecuteMsg::AcceptLiquidityRequest {} => {
            let action_type = ActionTypes::AcceptLiquidityRequest {};
            authorize(&deps, _info.sender.clone(), action_type)?;
            execute_accept_liquidity_request(deps, _env, &_info)
        }

        ExecuteMsg::ClaimDelegatorRewards {} => {
            let action_type = ActionTypes::ClaimDelegatorRewards {};
            authorize(&deps, _info.sender.clone(), action_type)?;
            execute_claim_delegator_rewards(deps, _env)
        }

        ExecuteMsg::RepayLoan {} => {
            let action_type = ActionTypes::RepayLoan(helpers::has_open_liquidity_request(&deps)?);
            authorize(&deps, _info.sender.clone(), action_type)?;
            execute_repay_loan(deps, _env)
        }

        ExecuteMsg::LiquidateCollateral {} => {
            let action_type =
                ActionTypes::LiquidateCollateral(helpers::has_open_liquidity_request(&deps)?);
            authorize(&deps, _info.sender.clone(), action_type)?;
            execute_liquidate_collateral(deps, _env)
        }

        ExecuteMsg::TransferOwnership { to_address } => {
            authorize(
                &deps,
                _info.sender.clone(),
                ActionTypes::TransferOwnership {},
            )?;
            execute_transfer_ownership(deps, to_address)
        }

        ExecuteMsg::Vote { proposal_id, vote } => {
            authorize(&deps, _info.sender.clone(), ActionTypes::Vote {})?;
            execute_vote(deps, _env, &_info, proposal_id, vote)
        }

        ExecuteMsg::WithdrawBalance { to_address, funds } => {
            authorize(&deps, _info.sender.clone(), ActionTypes::WithdrawBalance)?;
            execute_withdraw_balance(deps, _env, to_address, funds)
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
    // Init response object
    let mut response = Response::new();

    // Ensure validator is active
    helpers::ensure_validator_is_active(&deps, validator.as_str())?;

    // Verify that owner can still delegate if there is an active liquidity request
    helpers::can_delegate_with_active_liquidity_request(&deps, &env)?;

    // Validate amount to delegate is not above availabe contract balance
    let denom_str = deps.querier.query_bonded_denom()?;
    helpers::validate_amount_to_delegate(&env, &deps, amount, denom_str.clone())?;

    // Process lender claims on claimed accumulated staking rewards from validator
    if let Some(ActiveOption {
        msg: _,
        lender: Some(lender),
        state: Some(state),
    }) = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?
    {
        let (accumulated_rewards, distribute_msgs) =
            helpers::accumulated_rewards(&deps, &env, Some(vec![validator.clone()]))?;

        if !accumulated_rewards.is_zero() {
            response = response.add_messages(distribute_msgs);

            // Add msg for sending claimed rewards to the lender
            if let Some(transfer_msgs) =
                helpers::process_lender_claims(deps, &env, state, lender, accumulated_rewards)?
            {
                response = response.add_message(transfer_msgs);
            }
        }
    }

    // Create sdk_msg for staking tokens
    let sdk_msg = StakingMsg::Delegate {
        validator: validator.clone(),
        amount: Coin {
            denom: denom_str,
            amount,
        },
    };

    // Respond
    Ok(response.add_messages(vec![sdk_msg]).add_attributes(vec![
        attr("method", "delegate"),
        attr("amount", amount.to_string()),
        attr("validator", validator),
    ]))
}

pub fn execute_redelegate(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    src_validator: String,
    dst_validator: String,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let mut response = Response::new();
    let denom_str = deps.querier.query_bonded_denom()?;
    let config = CONFIG.load(deps.storage)?;

    // Ensure that the dst_validator is in the active set
    helpers::ensure_validator_is_active(&deps, dst_validator.as_str())?;

    // Allow the active lender to re-delegate away from inactive src_validator
    if info.sender.clone().ne(&config.owner) {
        helpers::ensure_lender_can_redelegate(&deps, src_validator.as_str())?;
    }

    // Process lender claims on claimed accumulated staking rewards from src_validator
    if let Some(ActiveOption {
        msg: _,
        lender: Some(lender),
        state: Some(state),
    }) = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?
    {
        let (accumulated_rewards, distribute_msgs) = helpers::accumulated_rewards(
            &deps,
            &env,
            Some(vec![src_validator.clone(), dst_validator.clone()]),
        )?;

        if !accumulated_rewards.is_zero() {
            response = response.add_messages(distribute_msgs);

            // Add msg for sending claimed rewards to the lender
            if let Some(transfer_msgs) =
                helpers::process_lender_claims(deps, &env, state, lender, accumulated_rewards)?
            {
                response = response.add_message(transfer_msgs);
            }
        }
    }

    // Create sdk_msg for re-delegating tokens
    let sdk_msg = StakingMsg::Redelegate {
        src_validator: src_validator.clone(),
        dst_validator: dst_validator.clone(),
        amount: Coin {
            denom: denom_str,
            amount,
        },
    };

    // Respond
    Ok(response.add_message(sdk_msg).add_attributes(vec![
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
    // Verify amount <= validator_delegation
    let validator_delegation = deps
        .querier
        .query_delegation(env.contract.address.clone(), validator.clone())
        .unwrap();
    if let Some(data) = validator_delegation {
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

pub fn execute_request_liquidity(
    deps: DepsMut,
    env: Env,
    option: LiquidityRequestMsg,
) -> Result<Response, ContractError> {
    // Validate liquidity request message to ensue that the correct data
    // was sent by the caller
    match option.clone() {
        LiquidityRequestMsg::FixedInterestRental {
            requested_amount,
            claimable_tokens,
            can_cast_vote: _,
        } => {
            if requested_amount.amount.is_zero() || claimable_tokens.is_zero() {
                return Err(ContractError::InvalidLiquidityRequestOption {});
            }
        }

        LiquidityRequestMsg::FixedTermRental {
            requested_amount,
            duration_in_seconds,
            can_cast_vote: _,
        } => {
            if requested_amount.amount.is_zero() || duration_in_seconds == 0u64 {
                return Err(ContractError::InvalidLiquidityRequestOption {});
            }
        }

        LiquidityRequestMsg::FixedTermLoan {
            requested_amount,
            duration_in_seconds,
            collateral_amount,
            interest_amount: _,
        } => {
            if helpers::query_total_delegations(&deps, &env)? < collateral_amount
                || collateral_amount.is_zero()
                || requested_amount.amount.is_zero()
                || duration_in_seconds == 0u64
            {
                return Err(ContractError::InvalidLiquidityRequestOption {});
            }
        }
    };

    // Save liquidity request message
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

pub fn execute_close_pending_liquidity_request(deps: DepsMut) -> Result<Response, ContractError> {
    // If the liquidity request is already active, we return LiquidityRequestIsActive
    if let Some(ActiveOption {
        msg: _,
        lender: Some(_lender),
        state: Some(_state),
    }) = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?
    {
        return Err(ContractError::LiquidityRequestIsActive {});
    }

    // Clear the pending liquidity request
    OPEN_LIQUIDITY_REQUEST.update(deps.storage, |mut _data| -> Result<_, ContractError> {
        Ok(None)
    })?;

    // respond
    Ok(Response::new().add_attributes(vec![attr("method", "close_liquidity_request")]))
}

pub fn execute_accept_liquidity_request(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
) -> Result<Response, ContractError> {
    let mut response = Response::new();
    let config = CONFIG.load(deps.storage)?;
    let (state, requested_amount) = helpers::map_liquidity_request_state(&deps, &env)?;

    // Verify that the lender is sending the correct requested amount
    helpers::validate_exact_input_amount(
        &info.funds,
        requested_amount.amount,
        requested_amount.denom.clone(),
    )?;

    // When the liquidity request option of type fixed term rental,
    // We claim all pending staking rewards for the vault owner before the option starts counting.
    if let LiquidityRequestState::FixedTermRental {
        requested_amount: _,
        start_time: _,
        end_time: _,
        last_claim_time: _,
        can_cast_vote: _,
    } = state
    {
        let (total_rewards_claimed, distribute_msgs) =
            helpers::accumulated_rewards(&deps, &env, None)?;
        if !total_rewards_claimed.is_zero() {
            response = response.add_messages(distribute_msgs);
        }
    }

    // Update state
    OPEN_LIQUIDITY_REQUEST.update(deps.storage, |data| -> Result<_, ContractError> {
        let mut option = data.unwrap();

        // Update the state
        option.state = Some(state);

        // Update the lender info
        option.lender = Some(info.sender.clone());

        Ok(Some(option))
    })?;

    // Add message to transfer liquidity request comission to INSTANTIATOR_ADDR
    let transfer_msg = helpers::get_bank_transfer_to_msg(
        &Addr::unchecked(INSTANTIATOR_ADDR),
        &requested_amount.denom,
        helpers::get_liquidity_comission(requested_amount.amount)?,
    );

    // respond
    Ok(response.add_message(transfer_msg).add_attributes(vec![
        attr("method", "accept_liquidity_request"),
        attr("amount", requested_amount.amount.to_string()),
        attr("vault_owner", config.owner.to_string()),
    ]))
}

pub fn execute_claim_delegator_rewards(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    // Init response object
    let mut response = Response::new();

    // Calculate total_rewards_claimed
    let (total_rewards_claimed, distribute_msgs) = helpers::accumulated_rewards(&deps, &env, None)?;
    if !total_rewards_claimed.is_zero() {
        response = response.add_messages(distribute_msgs);
    }

    // Process lender claims if there is an active rental option on the vault
    if let Some(ActiveOption {
        msg: _,
        lender: Some(lender),
        state: Some(state),
    }) = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?
    {
        if let Some(transfer_msgs) =
            helpers::process_lender_claims(deps, &env, state, lender, total_rewards_claimed)?
        {
            // Add msg for sending claimed rewards to the lender
            response = response.add_message(transfer_msgs);
        }
    }

    // respond
    Ok(response
        .add_attribute("method", "claim_delegator_rewards")
        .add_attribute("total_rewards_claimed", total_rewards_claimed.to_string()))
}

pub fn execute_repay_loan(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    // Init response object
    let mut response = Response::new();

    // If the lender has already triggered a liquidation event, the vault owner can instead
    // call liquidate_collateral to pay-off the outstanding debt with the free vault balance
    if let Some(ActiveOption {
        msg: _,
        lender: Some(lender),
        state:
            Some(LiquidityRequestState::FixedTermLoan {
                requested_amount,
                interest_amount,
                collateral_amount: _,
                start_time: _,
                end_time: _,
                last_liquidation_date: _,
                already_claimed: _,
                processing_liquidation: false,
            }),
    }) = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?
    {
        // Check if there is enough balance to repay requested_amount + interest_amount
        let repayment_amount = requested_amount.amount + interest_amount;
        let borrowed_denom_balance =
            helpers::get_balace_for_demon(&deps, &env, requested_amount.denom.clone())?;
        if borrowed_denom_balance.amount < repayment_amount {
            return Err(ContractError::InsufficientBalance {
                required: Coin {
                    amount: repayment_amount,
                    denom: requested_amount.denom.clone(),
                },
                available: Coin {
                    amount: borrowed_denom_balance.amount,
                    denom: requested_amount.denom.clone(),
                },
            });
        }

        // Add funds_transfer_msg to send repayment_amount to the lender
        response = response.add_message(helpers::get_bank_transfer_to_msg(
            &lender,
            &requested_amount.denom.clone(),
            repayment_amount,
        ));

        // Close option as repayment has been processed successfully
        OPEN_LIQUIDITY_REQUEST.update(deps.storage, |mut _data| -> Result<_, ContractError> {
            Ok(None)
        })?;
    } else {
        return Err(ContractError::Unauthorized {});
    }

    // respond
    Ok(response.add_attribute("method", "repay_loan"))
}

pub fn execute_liquidate_collateral(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let mut response = Response::new();

    // Check if there is an active FixedTermLoan loan on the vault
    if let Some(ActiveOption {
        msg: _,
        lender: Some(lender),
        state:
            Some(LiquidityRequestState::FixedTermLoan {
                requested_amount,
                interest_amount,
                collateral_amount,
                start_time,
                end_time,
                already_claimed,
                last_liquidation_date,
                processing_liquidation: _,
            }),
    }) = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?
    {
        // liquidation on fixed term loans can only happen on/after expiration date
        // TODO make this error message more descriptive
        if env.block.time < end_time {
            return Err(ContractError::Unauthorized {});
        }

        // Get available collateral balance
        let denom_str = deps.querier.query_bonded_denom()?;
        let available_collateral_balance =
            helpers::get_balace_for_demon(&deps, &env, denom_str.clone())?;

        // Get available staking rewards
        let (total_rewards_claimed, distribute_msgs) =
            helpers::accumulated_rewards(&deps, &env, None)?;

        // Calculate total available collateral balance
        let total_available_collateral_balance =
            available_collateral_balance.amount + total_rewards_claimed;

        // Calculate amount_to_claim which is limited by total_available_collateral_balance
        let outstanding_debt = collateral_amount - already_claimed;
        let amount_to_claim = if outstanding_debt < total_available_collateral_balance {
            outstanding_debt
        } else {
            total_available_collateral_balance
        };

        // Calculate duration_since_last_liquidation
        let duration_since_last_liquidation = if last_liquidation_date.is_some() {
            env.block.time.seconds() - last_liquidation_date.unwrap().seconds()
        } else {
            STAKE_LIQUIDATION_INTERVAL
        };

        // Add messages to unbond outstanding collateral amount from staked tokens
        // When total_available_collateral_balance is not enough to clear the debt
        let updated_already_claimed = already_claimed + amount_to_claim;
        let claims_not_completed = updated_already_claimed < collateral_amount;
        let can_unstake = duration_since_last_liquidation >= STAKE_LIQUIDATION_INTERVAL;
        let mut updated_last_liquidation_date = last_liquidation_date;
        if claims_not_completed && can_unstake {
            let undelegate_msgs = helpers::unbond_tokens_from_validators(
                &deps,
                &env,
                collateral_amount - updated_already_claimed,
            )?;

            if !undelegate_msgs.is_empty() {
                updated_last_liquidation_date = Some(env.block.time);
                response = response.add_messages(undelegate_msgs);
            }
        }

        // Add messages to claim available staking rewards
        if !total_rewards_claimed.is_zero() {
            response = response.add_messages(distribute_msgs);
        }

        // Add messages to send amount_to_claim to the lender
        if !amount_to_claim.is_zero() {
            response = response.add_message(helpers::get_bank_transfer_to_msg(
                &lender,
                &denom_str,
                amount_to_claim,
            ));
        }

        // Update the liquidity request state
        OPEN_LIQUIDITY_REQUEST.update(deps.storage, |data| -> Result<_, ContractError> {
            if claims_not_completed {
                let mut option = data.unwrap();
                option.state = Some(LiquidityRequestState::FixedTermLoan {
                    requested_amount,
                    interest_amount,
                    collateral_amount,
                    start_time,
                    end_time,
                    last_liquidation_date: updated_last_liquidation_date,
                    already_claimed: updated_already_claimed,
                    processing_liquidation: true,
                });

                Ok(Some(option))
            } else {
                // Close option as repayment has been processed successfully
                Ok(None)
            }
        })?;
    } else {
        return Err(ContractError::Unauthorized {});
    }

    // respond
    Ok(response.add_attribute("method", "liquidate_collateral"))
}

pub fn execute_withdraw_balance(
    deps: DepsMut,
    env: Env,
    to_address: Option<String>,
    funds: Coin,
) -> Result<Response, ContractError> {
    // Check if the contract balance is >= the requested amount to withdraw
    let available_balance = helpers::get_balace_for_demon(&deps, &env, funds.denom.clone())?;
    if available_balance.amount < funds.amount {
        return Err(ContractError::InsufficientBalance {
            available: Coin {
                amount: available_balance.amount,
                denom: funds.denom.clone(),
            },
            required: funds,
        });
    }

    // Check if user is trying to withdraw staking balance, as it is the token used as collateral,
    // we also check to make sure there is no outstandinding debt from a defaulted fixed term loan
    // on the vault else we return ContractError::ClearOutstandingDebt {amount: outstanding_amount}
    let staking_denom = deps.querier.query_bonded_denom()?;
    let outstanding_debt = helpers::outstanding_fixed_term_loan_debt(&deps, &env)?;
    if staking_denom.eq(&funds.denom.clone()) && outstanding_debt.gt(&Uint128::zero()) {
        return Err(ContractError::ClearOutstandingDebt {
            amount: Coin {
                amount: outstanding_debt,
                denom: staking_denom,
            },
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
    to_address: String,
) -> Result<Response, ContractError> {
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

// Test on testnet, until we figure out how to create
// a test proposal using multi-test
pub fn execute_vote(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    proposal_id: u64,
    vote: VoteOption,
) -> Result<Response, ContractError> {
    let mut response = Response::new();
    let config = CONFIG.load(deps.storage)?;
    let lender_can_cast_vote = helpers::current_lender_can_cast_vote(&deps, &env)?;

    // Check if owner can cast vote
    let owner_can_vote = info.sender.eq(&config.owner) && !lender_can_cast_vote;

    // Check if lender can cast vote
    let lender_can_vote = !info.sender.eq(&config.owner) && lender_can_cast_vote;

    // Add sdk_msg to vote
    if owner_can_vote || lender_can_vote {
        response = response.add_message(GovMsg::Vote { proposal_id, vote });
    }

    // respond
    Ok(response.add_attribute("method", "vote"))
}

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Info {} => to_binary(&query_info(deps)?),
        QueryMsg::StakingInfo {} => to_binary(&query_staking_info(deps, env)?),
        QueryMsg::AllDelegations {} => to_binary(&query_all_delegations(deps, env)?),
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

pub fn query_staking_info(deps: Deps, env: Env) -> StdResult<StakingInfoResponse> {
    let (total_staked, accumulated_rewards) = helpers::query_staking_info(&deps, &env)?;

    Ok(StakingInfoResponse {
        total_staked,
        accumulated_rewards,
    })
}

pub fn query_all_delegations(deps: Deps, env: Env) -> StdResult<AllDelegationsResponse> {
    let data = helpers::query_all_delegations(&deps, &env)?;
    Ok(AllDelegationsResponse { data })
}
