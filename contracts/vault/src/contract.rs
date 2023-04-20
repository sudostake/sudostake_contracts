use crate::authorisation::{authorize, ActionTypes};
use crate::error::ContractError;
use crate::helpers;
use crate::msg::{ExecuteMsg, InfoResponse, InstantiateMsg, LiquidityRequestOptionMsg, QueryMsg};
use crate::state::{
    ActiveOption, Config, LiquidityRequestOptionState, CONFIG, OPEN_LIQUIDITY_REQUEST,
};
use cosmwasm_std::{
    attr, entry_point, to_binary, Addr, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response,
    StakingMsg, StdResult, Uint128, VoteOption,
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

    // Set default liquidation interval for defaulted fixed term loans
    // Here we are setting as 30days
    // @TODO extract this into an oracle contract
    let liquidation_interval_in_seconds = 60 * 60 * 24 * 30;

    // Save contract state
    CONFIG.save(
        deps.storage,
        &Config {
            owner,
            acc_manager,
            liquidation_interval_in_seconds,
        },
    )?;

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
                ActionTypes::Delegate(helpers::has_open_liquidity_request(&deps)?),
            )?;
            execute_delegate(deps, _env, &_info, validator, amount)
        }

        ExecuteMsg::Undelegate { validator, amount } => {
            authorize(
                &deps,
                _info.sender.clone(),
                ActionTypes::Undelegate(helpers::has_open_liquidity_request(&deps)?),
            )?;
            execute_undelegate(deps, _env, &_info, validator, amount)
        }
        ExecuteMsg::OpenLiquidityRequest { option } => {
            authorize(
                &deps,
                _info.sender.clone(),
                ActionTypes::OpenLiquidityRequest(helpers::has_open_liquidity_request(&deps)?),
            )?;
            execute_open_liquidity_request(deps, _env, option)
        }
        ExecuteMsg::CloseLiquidityRequest {} => {
            authorize(
                &deps,
                _info.sender.clone(),
                ActionTypes::CloseLiquidityRequest(helpers::has_open_liquidity_request(&deps)?),
            )?;
            execute_close_liquidity_request(deps)
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
                ActionTypes::LiquidateCollateral(helpers::has_open_liquidity_request(&deps)?),
            )?;
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
        ExecuteMsg::Redelegate {
            src_validator,
            dst_validator,
            amount,
        } => {
            authorize(&deps, _info.sender.clone(), ActionTypes::Redelegate {})?;
            execute_redelegate(deps, _env, &_info, src_validator, dst_validator, amount)
        }

        ExecuteMsg::AcceptLiquidityRequest {} => {
            authorize(
                &deps,
                _info.sender.clone(),
                ActionTypes::AcceptLiquidityRequest {},
            )?;
            execute_accept_liquidity_request(deps, _env, &_info)
        }

        ExecuteMsg::RepayLoan {} => {
            authorize(
                &deps,
                _info.sender.clone(),
                ActionTypes::RepayLoan(helpers::has_open_liquidity_request(&deps)?),
            )?;
            execute_repay_loan(deps, _env)
        }
        ExecuteMsg::WithdrawBalance { to_address, funds } => {
            authorize(
                &deps,
                _info.sender.clone(),
                ActionTypes::WithdrawBalance(helpers::has_open_liquidity_request(&deps)?),
            )?;
            execute_withdraw_balance(deps, _env, to_address, funds)
        }
        ExecuteMsg::Vote { proposal_id, vote } => {
            authorize(&deps, _info.sender.clone(), ActionTypes::Vote {})?;
            execute_vote(deps, _env, &_info, proposal_id, vote)
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
    helpers::verify_validator_is_active(&deps, validator.as_str())?;

    // Validate amount to delegate is not above availabe contract balance
    let denom_str = deps.querier.query_bonded_denom()?;
    helpers::validate_amount_to_delegate(&env, &deps, amount, denom_str.clone())?;

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
    helpers::verify_validator_is_active(&deps, dst_validator.as_str())?;

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
    // Validate liquidity request option to ensue that the correct data
    // was sent by the caller
    match option.clone() {
        LiquidityRequestOptionMsg::FixedInterestRental {
            requested_amount,
            claimable_tokens,
            can_cast_vote: _,
        } => {
            if requested_amount.amount.is_zero() || claimable_tokens.is_zero() {
                return Err(ContractError::InvalidLiquidityRequestOption {});
            }
        }

        LiquidityRequestOptionMsg::FixedTermRental {
            requested_amount,
            duration_in_seconds,
            can_cast_vote: _,
        } => {
            if requested_amount.amount.is_zero() || duration_in_seconds == 0u64 {
                return Err(ContractError::InvalidLiquidityRequestOption {});
            }
        }

        LiquidityRequestOptionMsg::FixedTermLoan {
            requested_amount,
            interest_amount: _,
            duration_in_seconds,
            collateral_amount,
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

pub fn execute_close_liquidity_request(deps: DepsMut) -> Result<Response, ContractError> {
    // Check if liquidity reqest already has a lender connected
    let liquidity_request = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?.unwrap();
    if liquidity_request.lender.is_some() {
        return Err(ContractError::Unauthorized {});
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
    let option = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?.unwrap();

    // activate the open liquidity request by mapping
    // LiquidityRequestOptionMsg => LiquidityRequestOptionState
    match option.msg {
        LiquidityRequestOptionMsg::FixedInterestRental {
            requested_amount,
            claimable_tokens,
            can_cast_vote,
        } => {
            // verify that the lender is sending the correct requested amount
            let input_amount =
                helpers::get_amount_for_denom(&info.funds, requested_amount.denom.clone())?;
            if requested_amount.amount != input_amount {
                return Err(ContractError::InvalidInputAmount {
                    required: requested_amount.amount,
                    received: input_amount,
                });
            }

            // update state
            OPEN_LIQUIDITY_REQUEST.update(deps.storage, |data| -> Result<_, ContractError> {
                let mut option = data.unwrap();
                option.state = Some(LiquidityRequestOptionState::FixedInterestRental {
                    requested_amount,
                    claimable_tokens,
                    already_claimed: Uint128::zero(),
                    can_cast_vote,
                });

                // update the lender info
                option.lender = Some(info.sender.clone());
                Ok(Some(option))
            })?;
        }

        LiquidityRequestOptionMsg::FixedTermRental {
            requested_amount,
            duration_in_seconds,
            can_cast_vote,
        } => {
            // verify that the lender is sending the correct requested amount
            let input_amount =
                helpers::get_amount_for_denom(&info.funds, requested_amount.denom.clone())?;
            if requested_amount.amount != input_amount {
                return Err(ContractError::InvalidInputAmount {
                    required: requested_amount.amount,
                    received: input_amount,
                });
            }

            // update state
            OPEN_LIQUIDITY_REQUEST.update(deps.storage, |data| -> Result<_, ContractError> {
                let mut option = data.unwrap();
                option.state = Some(LiquidityRequestOptionState::FixedTermRental {
                    requested_amount,
                    start_time: env.block.time,
                    end_time: env.block.time.plus_seconds(duration_in_seconds),
                    last_claim_time: env.block.time,
                    can_cast_vote,
                });

                // update the lender info
                option.lender = Some(info.sender.clone());
                Ok(Some(option))
            })?;
        }

        LiquidityRequestOptionMsg::FixedTermLoan {
            requested_amount,
            interest_amount,
            duration_in_seconds,
            collateral_amount,
        } => {
            // verify that the lender is sending the correct requested amount
            let input_amount =
                helpers::get_amount_for_denom(&info.funds, requested_amount.denom.clone())?;
            if requested_amount.amount != input_amount {
                return Err(ContractError::InvalidInputAmount {
                    required: requested_amount.amount,
                    received: input_amount,
                });
            }

            // update state
            OPEN_LIQUIDITY_REQUEST.update(deps.storage, |data| -> Result<_, ContractError> {
                let mut option = data.unwrap();
                option.state = Some(LiquidityRequestOptionState::FixedTermLoan {
                    requested_amount,
                    interest_amount,
                    collateral_amount,
                    start_time: env.block.time,
                    end_time: env.block.time.plus_seconds(duration_in_seconds),
                    last_liquidation_date: None,
                    already_claimed: Uint128::zero(),
                    processing_liquidation: false,
                });

                // update the lender info
                option.lender = Some(info.sender.clone());
                Ok(Some(option))
            })?;
        }
    };

    // respond
    Ok(Response::new().add_attributes(vec![attr("method", "accept_liquidity_request")]))
}

pub fn execute_claim_delegator_rewards(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    // Calculate total_rewards_claimed and build distribute_msgs
    let mut response = Response::new();
    let (total_rewards_claimed, distribute_msgs) =
        helpers::calculate_total_claimed_rewards(&deps, &env)?;
    if !total_rewards_claimed.is_zero() {
        response = response.add_messages(distribute_msgs);
    }

    // Process lender claims if there is an active liquidity request on the vault
    // then add the transfer_msg for lender_claims to response
    if let Some(ActiveOption {
        msg: _,
        lender: Some(lender),
        state: Some(state),
    }) = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?
    {
        let lender_claims =
            helpers::process_lender_claims(deps, &env, state, lender, total_rewards_claimed)?;
        if let Some(msg) = lender_claims {
            // Add msg for sending claimed rewards to the lender
            response = response.add_message(msg);
        }
    }

    // respond
    Ok(response
        .add_attribute("method", "claim_delegator_rewards")
        .add_attribute("total_rewards_claimed", total_rewards_claimed.to_string()))
}

pub fn execute_repay_loan(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    let liquidity_request = OPEN_LIQUIDITY_REQUEST.load(deps.storage)?;
    let mut response = Response::new();

    // Check if there is an active FixedTermLoan loan on the vault
    // where processing_liquidation is false.
    if let Some(ActiveOption {
        msg: _,
        lender: Some(lender),
        state:
            Some(LiquidityRequestOptionState::FixedTermLoan {
                requested_amount,
                interest_amount,
                collateral_amount: _,
                start_time: _,
                end_time: _,
                last_liquidation_date: _,
                already_claimed: _,
                processing_liquidation: false,
            }),
    }) = liquidity_request
    {
        // Check if there is enough balance to repay requested_amount + interest_amount
        let repayment_amount = requested_amount.amount + interest_amount;
        let borrowed_denom_balance = helpers::get_amount_for_denom(
            &deps
                .querier
                .query_all_balances(env.contract.address.clone())?,
            requested_amount.denom.clone(),
        )?;
        if borrowed_denom_balance < repayment_amount {
            return Err(ContractError::InsufficientBalance {
                required: Coin {
                    amount: repayment_amount,
                    denom: requested_amount.denom.clone(),
                },
                available: Coin {
                    amount: borrowed_denom_balance,
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
            Some(LiquidityRequestOptionState::FixedTermLoan {
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
        // liquidation on fixed term loans can only happen after expiration date
        if env.block.time < end_time {
            return Err(ContractError::Unauthorized {});
        }

        // Calculate total_available_collateral_balance = available_collateral_balance + total_rewards_claimed
        let config = CONFIG.load(deps.storage)?;
        let denom_str = deps.querier.query_bonded_denom()?;
        let available_collateral_balance = helpers::get_amount_for_denom(
            &deps
                .querier
                .query_all_balances(env.contract.address.clone())?,
            denom_str.to_string(),
        )?;
        let (total_rewards_claimed, distribute_msgs) =
            helpers::calculate_total_claimed_rewards(&deps, &env)?;
        let total_available_collateral_balance =
            available_collateral_balance + total_rewards_claimed;

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
            config.liquidation_interval_in_seconds
        };

        // Add messages to unbond outstanding collateral amount from staked tokens
        // When total_available_collateral_balance is not enough to clear the debt
        let updated_already_claimed = already_claimed + amount_to_claim;
        let claims_not_completed = updated_already_claimed < collateral_amount;
        let can_unstake = duration_since_last_liquidation >= config.liquidation_interval_in_seconds;
        let mut updated_last_liquidation_date = last_liquidation_date;
        if claims_not_completed && can_unstake {
            if let Some(undelegate_msgs) = helpers::unbond_tokens_from_validators(
                &deps,
                &env,
                collateral_amount - updated_already_claimed,
            )? {
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
                option.state = Some(LiquidityRequestOptionState::FixedTermLoan {
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
    let recipient: Addr;
    if let Some(val) = to_address {
        recipient = deps.api.addr_validate(&val)?;
    } else {
        let config = CONFIG.load(deps.storage)?;
        recipient = config.owner;
    }

    // construct sdk msg to transfer funds to recipient
    let msg = helpers::get_bank_transfer_to_msg(&recipient, &funds.denom, funds.amount);

    Ok(Response::new().add_message(msg).add_attributes(vec![
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

pub fn execute_vote(
    deps: DepsMut,
    env: Env,
    info: &MessageInfo,
    proposal_id: u64,
    vote: VoteOption,
) -> Result<Response, ContractError> {
    let mut response = Response::new();
    // todo
    //
    // lender_can_cast_vote = has_active_lro ? (active_option_gives_voting_rights ? true : false) : false
    //
    // owner_can_vote = info.sender == owner && !lender_can_cast_vote
    //
    // lender_can_vote = info.sender == lender && lender_can_cast_vote
    //
    // if owner_can_vote || lender_can_vote {
    //    response = response.add_message(GovMsg::Vote { proposal_id, vote });
    // }
    //
    // respond
    Ok(response.add_attribute("method", "vote"))
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
